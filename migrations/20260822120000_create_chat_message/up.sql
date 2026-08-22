-- Historique de conversation persistant, support de deux lectures distinctes :
--
--  1. l'agent, via `ConversationMemory` de rig : il relit `payload`, le message rig serialise en
--     entier (appels d'outils et resultats inclus) — le modele en a besoin pour poursuivre un
--     tour interrompu par un outil ;
--  2. le front, via GET /api/chat-history/history/{id} : il lit `role` + `content`, la projection
--     texte plate attendue par ChatMessageHistory.
--
-- Stocker directement la forme aplatie casserait l'agent ; ne stocker que le payload obligerait
-- le endpoint d'historique a parser du JSON par ligne. Les deux colonnes coexistent donc.
CREATE TABLE IF NOT EXISTS chat_message (
    id UUID PRIMARY KEY,

    -- VARCHAR et non UUID, sans cle etrangere : le contrat de `ConversationMemory` donne un
    -- identifiant de conversation sous forme de chaine libre. Le typer en UUID ferait echouer la
    -- memoire sur un identifiant que nous n'avons pas emis nous-memes.
    conversation_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,

    -- Ordre au sein de la conversation. Indispensable : un tour complet (prompt, appel d'outil,
    -- resultat, reponse finale) est ecrit en une seule fois, donc `created_at` est identique pour
    -- les quatre messages et ne peut pas les ordonner.
    sequence INTEGER NOT NULL,

    role VARCHAR(20) NOT NULL,
    content TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL
);

-- L'ordre est (conversation, sequence) : c'est la lecture faite a chaque tour de l'agent.
CREATE UNIQUE INDEX IF NOT EXISTS chat_message_conversation_sequence_idx
    ON chat_message (conversation_id, sequence);

-- Liste des conversations de l'utilisateur, du plus recent au plus ancien.
CREATE INDEX IF NOT EXISTS chat_message_user_created_idx
    ON chat_message (user_id, created_at DESC);
