-- Le garde d'onboarding du front lit `onboardingCompleted` sur GET /api/account : sans la colonne,
-- la valeur serait toujours false et le wizard bouclerait indefiniment apres l'avoir termine.
ALTER TABLE users ADD COLUMN IF NOT EXISTS onboarding_completed BOOLEAN NOT NULL DEFAULT FALSE;
