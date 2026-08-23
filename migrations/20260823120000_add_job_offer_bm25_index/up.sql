-- Index BM25 (pg_search / ParadeDB) sur les offres, pour remplacer le classement lexical maison.
--
-- Ce que le fait maison n'avait pas, et qui est mesure sur le corpus emploi.nc + boards ATS :
--
--  1. Frontieres de mots. `unaccent(col) ILIKE '%rust%'` remontait 805 offres sur 960, dont 801
--     ne contenaient que « trust » (« Zero Trust », « trusted by »). BM25 en trouve 60, le nombre
--     reel d'offres ou « rust » est un mot.
--  2. Ponderation des termes (IDF). Un terme present dans 433 offres et un present dans 60 avaient
--     le meme poids ; BM25 donne 1,65 au premier et 4,55 au second.
--  3. Saturation de frequence et normalisation par longueur : une annonce verbeuse n'est plus
--     avantagee du seul fait qu'elle contient plus de texte.
--  4. Un index. `unaccent()` n'est pas IMMUTABLE, donc le filtre precedent imposait un Seq Scan,
--     d'ou un plafond de balayage a 500 lignes au-dela duquel les resultats etaient tronques avant
--     d'etre scores.
CREATE EXTENSION IF NOT EXISTS pg_search;

-- Configuration du tokenizer, etablie par mesure et non par defaut :
--
--  * `ascii_folding` est indispensable. Sans lui, « developpeur » ne remonte RIEN alors que
--    « développeur » remonte 5 offres — la recherche sans accent, que la plupart des utilisateurs
--    font, serait cassee. Attention : pg_search accepte silencieusement une option inconnue, et
--    `ascii_folding` place au niveau du champ (et non du tokenizer) est ignore sans erreur.
--
--  * PAS de stemmer, volontairement. Avec `"stemmer": "French"` sur ce corpus majoritairement
--    anglophone, « sécurité » passait de 14 a 812 resultats : la racine « secur » attrapait
--    security, secure, securing… Le stemmer francais fait gagner les pluriels francais et perdre
--    toute precision sur l'anglais. Un corpus bilingue demanderait un champ par langue ; le
--    compromis retenu est de n'en mettre aucun.
--
-- `search_text` contient deja titre + entreprise + lieu + competences + description (concatenes a
-- l'insertion), donc l'interroger seul suffit au rappel. `title` et `company` sont indexes en plus
-- pour permettre plus tard des requetes ciblees ou une ponderation de champ.
CREATE INDEX IF NOT EXISTS job_offer_bm25_idx ON job_offer
USING bm25 (id, title, company, search_text)
WITH (
    key_field = 'id',
    text_fields = '{
        "title":       {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"},
        "company":     {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"},
        "search_text": {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"}
    }'
);
