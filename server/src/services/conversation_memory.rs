//! Historique de conversation persistant, branche sur rig via [`ConversationMemory`].
//!
//! rig fournit le trait et un backend en memoire de processus (`InMemoryConversationMemory`),
//! perdu au redemarrage. Cette implementation le remplace par Postgres, et sert deux lectures :
//!
//!  - l'agent, qui relit le message rig complet (appels d'outils et resultats inclus) ;
//!  - `GET /api/chat-history/history/{id}`, qui lit la projection texte plate attendue par le
//!    front.
//!
//! Une fois cet objet passe a `AgentBuilder::memory`, rig prend en charge le reste : chargement
//! avant le prompt, ecriture du tour complet apres succes, une seule fois par run meme quand le
//! tour a comporte plusieurs appels au modele, et rien du tout en cas d'echec.

use std::sync::Arc;

use chrono::Utc;
use diesel::prelude::*;
use rig::completion::Message;
use rig::completion::message::{AssistantContent, UserContent};
use rig::memory::{ConversationMemory, MemoryError};
use rig::wasm_compat::WasmBoxedFuture;
use uuid::Uuid;

use crate::db::connection::DbPool;
use crate::db::schema::chat_message;
use crate::models::chat_message::{chat_role, ChatMessage, NewChatMessage};

/// Historique de conversation stocke dans Postgres.
///
/// `user_id` est porte par l'instance et non par la methode : la signature du trait rig ne
/// transporte qu'un identifiant de conversation. On construit donc une memoire par utilisateur,
/// ce qui a l'avantage de rendre l'isolation structurelle plutot que dependante d'un filtre
/// qu'on pourrait oublier.
#[derive(Clone)]
pub struct PostgresConversationMemory {
    pool: DbPool,
    user_id: Arc<str>,
}

impl PostgresConversationMemory {
    pub fn new(pool: DbPool, user_id: impl AsRef<str>) -> Self {
        Self {
            pool,
            user_id: Arc::from(user_id.as_ref()),
        }
    }

    /// Messages d'une conversation, dans l'ordre, sous leur forme persistee.
    ///
    /// Utilise a la fois par le chargement de la memoire et par les endpoints d'historique, pour
    /// que les deux voient exactement la meme sequence.
    pub fn rows(
        conn: &mut crate::db::DbConnection,
        user_id: &str,
        conversation_id: &str,
    ) -> Result<Vec<ChatMessage>, crate::errors::AppError> {
        Ok(chat_message::table
            .filter(chat_message::user_id.eq(user_id))
            .filter(chat_message::conversation_id.eq(conversation_id))
            // Par `sequence` et non par `created_at` : un tour complet est ecrit en une fois, donc
            // tous ses messages partagent le meme horodatage.
            .order(chat_message::sequence.asc())
            .select(ChatMessage::as_select())
            .load(conn)?)
    }

    /// Toutes les conversations de l'utilisateur, messages confondus, du plus ancien au plus
    /// recent a l'interieur de chaque conversation.
    pub fn all_rows_for_user(
        conn: &mut crate::db::DbConnection,
        user_id: &str,
    ) -> Result<Vec<ChatMessage>, crate::errors::AppError> {
        Ok(chat_message::table
            .filter(chat_message::user_id.eq(user_id))
            .order((
                chat_message::created_at.asc(),
                chat_message::conversation_id.asc(),
                chat_message::sequence.asc(),
            ))
            .select(ChatMessage::as_select())
            .load(conn)?)
    }
}

impl ConversationMemory for PostgresConversationMemory {
    fn load<'a>(
        &'a self,
        conversation_id: &'a str,
    ) -> WasmBoxedFuture<'a, Result<Vec<Message>, MemoryError>> {
        let pool = self.pool.clone();
        let user_id = self.user_id.clone();
        let conversation_id = conversation_id.to_owned();

        Box::pin(async move {
            // Diesel est synchrone : sans `spawn_blocking`, la requete bloquerait un worker du
            // runtime Tokio pendant tout son aller-retour reseau.
            let rows = tokio::task::spawn_blocking(move || {
                let mut conn = pool.get().map_err(MemoryError::backend)?;
                Self::rows(&mut conn, &user_id, &conversation_id)
                    .map_err(|e| MemoryError::Backend(e.to_string().into()))
            })
            .await
            .map_err(|e| MemoryError::Internal(e.to_string()))??;

            rows.into_iter()
                .map(|row| {
                    serde_json::from_str::<Message>(&row.payload).map_err(|e| {
                        // Un payload illisible n'est pas ignorable en silence : renvoyer un
                        // historique tronque ferait repondre le modele a cote sans que personne
                        // ne sache pourquoi.
                        MemoryError::Backend(
                            format!("Message {} illisible : {e}", row.id).into(),
                        )
                    })
                })
                .collect()
        })
    }

    fn append<'a>(
        &'a self,
        conversation_id: &'a str,
        messages: Vec<Message>,
    ) -> WasmBoxedFuture<'a, Result<(), MemoryError>> {
        let pool = self.pool.clone();
        let user_id = self.user_id.clone();
        let conversation_id = conversation_id.to_owned();

        Box::pin(async move {
            if messages.is_empty() {
                return Ok(());
            }

            // Serialisation avant d'entrer dans la tache bloquante : un echec ici est une erreur
            // de programmation, pas une panne de base, et il ne doit pas laisser un tour a moitie
            // ecrit derriere lui.
            let encoded = messages
                .iter()
                .map(|message| {
                    let payload = serde_json::to_string(message)
                        .map_err(|e| MemoryError::Backend(e.to_string().into()))?;
                    Ok((role_of(message).to_string(), text_of(message), payload))
                })
                .collect::<Result<Vec<(String, String, String)>, MemoryError>>()?;

            tokio::task::spawn_blocking(move || {
                let mut conn = pool.get().map_err(MemoryError::backend)?;
                let now = Utc::now().naive_utc();

                // Transaction : le calcul de la sequence suivante et les insertions doivent etre
                // atomiques, sinon deux tours concurrents sur la meme conversation se disputeraient
                // le meme numero — que l'index unique refuserait.
                conn.transaction::<(), diesel::result::Error, _>(|conn| {
                    let next: i32 = chat_message::table
                        .filter(chat_message::conversation_id.eq(&conversation_id))
                        .select(diesel::dsl::max(chat_message::sequence))
                        .first::<Option<i32>>(conn)?
                        .map(|max| max + 1)
                        .unwrap_or(0);

                    let rows: Vec<NewChatMessage> = encoded
                        .into_iter()
                        .enumerate()
                        .map(|(offset, (role, content, payload))| NewChatMessage {
                            id: Uuid::new_v4(),
                            conversation_id: conversation_id.clone(),
                            user_id: user_id.to_string(),
                            sequence: next + offset as i32,
                            role,
                            content,
                            payload,
                            created_at: now,
                        })
                        .collect();

                    diesel::insert_into(chat_message::table)
                        .values(&rows)
                        .execute(conn)?;
                    Ok(())
                })
                .map_err(|e| MemoryError::Backend(e.to_string().into()))
            })
            .await
            .map_err(|e| MemoryError::Internal(e.to_string()))?
        })
    }

    fn clear<'a>(
        &'a self,
        conversation_id: &'a str,
    ) -> WasmBoxedFuture<'a, Result<(), MemoryError>> {
        let pool = self.pool.clone();
        let user_id = self.user_id.clone();
        let conversation_id = conversation_id.to_owned();

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                let mut conn = pool.get().map_err(MemoryError::backend)?;
                diesel::delete(
                    chat_message::table
                        .filter(chat_message::user_id.eq(user_id.as_ref()))
                        .filter(chat_message::conversation_id.eq(&conversation_id)),
                )
                .execute(&mut conn)
                .map_err(|e| MemoryError::Backend(e.to_string().into()))?;
                Ok(())
            })
            .await
            .map_err(|e| MemoryError::Internal(e.to_string()))?
        })
    }
}

/// Role persiste d'un message rig.
pub fn role_of(message: &Message) -> &'static str {
    match message {
        Message::System { .. } => chat_role::SYSTEM,
        Message::User { .. } => chat_role::USER,
        Message::Assistant { .. } => chat_role::ASSISTANT,
    }
}

/// Projection texte d'un message rig, pour la colonne `content` et donc pour le front.
///
/// Seules les parties textuelles sont retenues. Un message qui ne porte qu'un appel d'outil ou
/// qu'un resultat d'outil n'a pas de texte : il produit une chaine vide, et c'est l'endpoint
/// d'historique qui decide de ne pas l'afficher. L'information n'est pas perdue pour autant —
/// `payload` la conserve pour l'agent.
pub fn text_of(message: &Message) -> String {
    match message {
        Message::System { content } => content.clone(),
        Message::User { content } => content
            .iter()
            .filter_map(|item| match item {
                UserContent::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
            .collect::<Vec<&str>>()
            .join("\n"),
        Message::Assistant { content, .. } => content
            .iter()
            .filter_map(|item| match item {
                AssistantContent::Text(text) => Some(text.text.as_str()),
                _ => None,
            })
            .collect::<Vec<&str>>()
            .join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_of_maps_every_rig_variant() {
        assert_eq!(role_of(&Message::user("bonjour")), chat_role::USER);
        assert_eq!(role_of(&Message::assistant("salut")), chat_role::ASSISTANT);
        assert_eq!(
            role_of(&Message::System { content: "consigne".into() }),
            chat_role::SYSTEM
        );
    }

    #[test]
    fn text_of_extracts_the_textual_parts() {
        assert_eq!(text_of(&Message::user("bonjour")), "bonjour");
        assert_eq!(text_of(&Message::assistant("salut")), "salut");
        assert_eq!(
            text_of(&Message::System { content: "consigne".into() }),
            "consigne"
        );
    }

    #[test]
    fn payload_round_trips_without_loss() {
        // C'est la garantie qui fait tenir tout le stockage : ce que l'agent relit doit etre
        // exactement ce qu'il a ecrit, sinon un tour interrompu par un outil repart faux.
        for message in [
            Message::user("bonjour"),
            Message::assistant("salut"),
            Message::System { content: "consigne".into() },
        ] {
            let encoded = serde_json::to_string(&message).unwrap();
            let decoded: Message = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, message, "aller-retour non fidele pour {message:?}");
        }
    }

    #[test]
    fn payload_keeps_the_role_tag_readable() {
        // Le tag `role` de rig est ce qui permet de relire un payload sans connaitre le contexte.
        let encoded = serde_json::to_string(&Message::user("bonjour")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(value.get("role").and_then(|r| r.as_str()), Some("user"));
    }

    #[test]
    fn memory_is_usable_as_a_rig_conversation_memory() {
        // Preuve a la compilation que le type se passe a `AgentBuilder::memory`, y compris sous
        // forme de trait objet.
        fn accepts<M: ConversationMemory + 'static>() {}
        accepts::<PostgresConversationMemory>();
        fn accepts_dyn(_: Box<dyn ConversationMemory>) {}
        let _ = accepts_dyn;
    }
}
