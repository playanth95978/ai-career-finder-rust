-- Recherche insensible aux accents. Sans elle, « developpeur » ne trouve aucune des offres
-- intitulees « Développeur » : la quasi-totalite du corpus emploi.nc devient introuvable des que
-- l'utilisateur tape sans accents, ce que font la plupart des gens dans une barre de recherche.
CREATE EXTENSION IF NOT EXISTS unaccent;
