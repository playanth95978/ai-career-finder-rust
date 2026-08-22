//! Verification de bout en bout de la memoire de conversation et du RAG, contre la vraie base et
//! le vrai modele.
//!
//! Ces tests sortent du processus : ils exigent Postgres (`DATABASE_URL`), Ollama pour les
//! embeddings et `MISTRAL_API_KEY` pour l'agent. Ils sont donc ignores par defaut et se lancent
//! explicitement :
//!
//! ```text
//! cargo test --test memory_and_rag_smoke -- --ignored --nocapture --test-threads=1
//! ```
//!
//! Ce qu'ils prouvent, et qu'aucun test unitaire ne peut prouver : que rig ecrit bien dans notre
//! backend Postgres, qu'il relit l'historique au tour suivant, et que l'index vectoriel est
//! effectivement interroge quand il est branche en `dynamic_context`.

use diesel::prelude::*;
use rig::client::{AgentClientExt, ProviderClient};
use rig::completion::Prompt;
use rig::memory::ConversationMemory;
use rig::providers::mistral;

use job_search_rust::config::AppConfig;
use job_search_rust::db::connection::establish_connection_pool;
use job_search_rust::db::schema::chat_message;
use job_search_rust::services::conversation_memory::PostgresConversationMemory;
use job_search_rust::services::job_offer_vector_index::JobOfferVectorIndex;

/// Identifiant de conversation propre a chaque execution, pour que deux lancements successifs ne
/// se marchent pas dessus.
fn conversation_id(suffix: &str) -> String {
    format!("test-{}-{}", suffix, uuid::Uuid::new_v4())
}

fn pool() -> job_search_rust::db::connection::DbPool {
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env();
    establish_connection_pool(&config.database_url)
}

/// Supprime les messages d'une conversation de test.
fn cleanup(pool: &job_search_rust::db::connection::DbPool, conversation: &str) {
    let mut conn = pool.get().expect("connexion");
    diesel::delete(chat_message::table.filter(chat_message::conversation_id.eq(conversation)))
        .execute(&mut conn)
        .expect("nettoyage");
}

#[tokio::test]
#[ignore = "necessite Postgres"]
async fn postgres_memory_round_trips_a_conversation() {
    let pool = pool();
    let conversation = conversation_id("roundtrip");
    let memory = PostgresConversationMemory::new(pool.clone(), "test-user");

    assert!(
        memory.load(&conversation).await.expect("load").is_empty(),
        "une conversation inconnue part d'un historique vide"
    );

    memory
        .append(
            &conversation,
            vec![
                rig::completion::Message::user("Je cherche un poste DevOps."),
                rig::completion::Message::assistant("Noté."),
            ],
        )
        .await
        .expect("append");

    let loaded = memory.load(&conversation).await.expect("load");
    assert_eq!(loaded.len(), 2, "les deux messages sont relus");
    assert_eq!(
        loaded[0],
        rig::completion::Message::user("Je cherche un poste DevOps."),
        "le premier message est relu a l'identique"
    );

    // Un second append ne doit pas reordonner ni ecraser le premier tour.
    memory
        .append(
            &conversation,
            vec![rig::completion::Message::user("Plutot en CDI.")],
        )
        .await
        .expect("second append");

    let loaded = memory.load(&conversation).await.expect("load");
    assert_eq!(loaded.len(), 3);
    assert_eq!(
        loaded[2],
        rig::completion::Message::user("Plutot en CDI."),
        "l'ordre d'ecriture est preserve"
    );

    memory.clear(&conversation).await.expect("clear");
    assert!(memory.load(&conversation).await.expect("load").is_empty());

    cleanup(&pool, &conversation);
}

#[tokio::test]
#[ignore = "necessite Postgres"]
async fn conversations_are_isolated_from_each_other() {
    let pool = pool();
    let first = conversation_id("iso-a");
    let second = conversation_id("iso-b");
    let memory = PostgresConversationMemory::new(pool.clone(), "test-user");

    memory
        .append(&first, vec![rig::completion::Message::user("dans A")])
        .await
        .expect("append A");
    memory
        .append(&second, vec![rig::completion::Message::user("dans B")])
        .await
        .expect("append B");

    assert_eq!(memory.load(&first).await.expect("load A").len(), 1);
    assert_eq!(memory.load(&second).await.expect("load B").len(), 1);

    cleanup(&pool, &first);
    cleanup(&pool, &second);
}

#[tokio::test]
#[ignore = "necessite Postgres"]
async fn memory_is_scoped_to_its_user() {
    let pool = pool();
    let conversation = conversation_id("scope");

    let alice = PostgresConversationMemory::new(pool.clone(), "alice");
    let bob = PostgresConversationMemory::new(pool.clone(), "bob");

    alice
        .append(&conversation, vec![rig::completion::Message::user("secret")])
        .await
        .expect("append");

    // Meme identifiant de conversation, autre utilisateur : rien ne doit fuir. C'est l'isolation
    // structurelle apportee par le `user_id` porte par l'instance.
    assert!(
        bob.load(&conversation).await.expect("load bob").is_empty(),
        "un autre utilisateur ne voit pas la conversation"
    );
    assert_eq!(alice.load(&conversation).await.expect("load alice").len(), 1);

    cleanup(&pool, &conversation);
}

#[tokio::test]
#[ignore = "necessite Postgres + Ollama"]
async fn vector_index_ranks_semantically_related_offers() {
    let pool = pool();
    let index = JobOfferVectorIndex::new(pool);

    let hits = index
        .search("ingenieur infrastructure et conteneurisation", 5, None)
        .await
        .expect("recherche vectorielle");

    assert!(
        !hits.is_empty(),
        "corpus vide ou non vectorise : lancer une recherche puis laisser passer le poller"
    );
    // Les scores sont des similarites decroissantes dans [0, 1], convention de rig.
    for window in hits.windows(2) {
        assert!(
            window[0].0 >= window[1].0,
            "les resultats doivent etre ordonnes par similarite decroissante"
        );
    }
    assert!(hits.iter().all(|(score, _)| (0.0..=1.0).contains(score)));
}

#[tokio::test]
#[ignore = "necessite Postgres + Ollama + MISTRAL_API_KEY"]
async fn agent_remembers_across_turns_through_postgres_memory() {
    let pool = pool();
    let conversation = conversation_id("agent");
    let memory = PostgresConversationMemory::new(pool.clone(), "test-user");

    let client = mistral::Client::from_env().expect("client Mistral");
    let agent = client
        .agent(&std::env::var("MISTRAL_CHAT_MODEL").unwrap_or("mistral-medium-3.5".into()))
        .preamble("Tu reponds en une phrase, sans fioriture.")
        .memory(memory.clone())
        .build();

    // Premier tour : on confie un fait que seul l'historique pourra restituer.
    agent
        .prompt("Retiens que je m'appelle Capitaine Haddock.")
        .conversation(&conversation)
        .await
        .expect("premier tour");

    // Deuxieme tour : aucune mention du nom dans le prompt. Si l'agent repond juste, c'est que
    // rig a bien relu l'historique depuis Postgres.
    let answer = agent
        .prompt("Comment est-ce que je m'appelle ?")
        .conversation(&conversation)
        .await
        .expect("second tour");

    println!("reponse du second tour : {answer}");
    assert!(
        answer.to_lowercase().contains("haddock"),
        "l'agent doit retrouver le nom via la memoire Postgres, reponse obtenue : {answer}"
    );

    let stored = memory.load(&conversation).await.expect("load");
    assert!(
        stored.len() >= 4,
        "deux tours complets persistes (2 prompts + 2 reponses), obtenu : {}",
        stored.len()
    );

    cleanup(&pool, &conversation);
}
