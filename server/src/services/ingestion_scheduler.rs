//! Declenchement planifie de l'ingestion des offres.
//!
//! Equivalent de `JobOfferBatchScheduler` cote Spring, avec la meme expression cron par defaut.
//! `tokio-cron-scheduler` plutot qu'un `tokio::time::interval` : l'intervalle donnerait « toutes
//! les vingt-quatre heures a partir du demarrage du serveur », ce qui fait deriver l'ingestion vers
//! les heures ouvrees a chaque redeploiement. Une ingestion de plusieurs milliers d'offres doit
//! tomber la nuit, pas quand elle veut.

use std::sync::Arc;

use tokio_cron_scheduler::{Job, JobScheduler};

use crate::config::ats_config::AtsConfig;
use crate::db::DbPool;
use crate::services::job_offer_ingestion_service::JobOfferIngestionService;

/// Expression par defaut : tous les jours a 2 h. Identique au `@Scheduled(cron = "0 0 2 * * *")`
/// du backend Java, pour que les deux ingerent au meme rythme si les deux tournent.
const DEFAULT_CRON: &str = "0 0 2 * * *";

/// Lance l'ordonnanceur d'ingestion.
///
/// Ne renvoie pas d'erreur : une expression cron invalide ou un ordonnanceur qui refuse de demarrer
/// ne doivent pas empecher le serveur de servir des requetes. L'ingestion est une alimentation de
/// fond, la recherche fonctionne sur le corpus deja en base sans elle.
pub async fn spawn(pool: DbPool, config: AtsConfig) {
    let expression = cron_expression();

    // Desactivation explicite : sur un poste de developpement, on ne veut pas qu'un serveur laisse
    // tourner declenche a 2 h quatorze requetes vers une API a quota.
    if expression.eq_ignore_ascii_case("off") || expression.eq_ignore_ascii_case("disabled") {
        tracing::info!("Ingestion planifiee desactivee (INGESTION_CRON)");
        return;
    }

    let scheduler = match JobScheduler::new().await {
        Ok(scheduler) => scheduler,
        Err(e) => {
            tracing::error!(error = %e, "Ordonnanceur d'ingestion indisponible");
            return;
        }
    };

    let context = Arc::new((pool, config));
    let job = match Job::new_async(expression.as_str(), move |_id, _lock| {
        let context = Arc::clone(&context);
        Box::pin(async move {
            let (pool, config) = context.as_ref();
            match JobOfferIngestionService::run(pool, config).await {
                Ok(summary) => tracing::info!(
                    read = summary.read,
                    written = summary.written,
                    failed = summary.partitions_failed,
                    "Ingestion planifiee terminee"
                ),
                // L'erreur est journalisee et non propagee : la tache suivante doit se declencher
                // meme si celle-ci n'a pas pu demarrer.
                Err(e) => tracing::error!(error = %e, "Ingestion planifiee en echec"),
            }
        })
    }) {
        Ok(job) => job,
        Err(e) => {
            tracing::error!(
                error = %e,
                expression = %expression,
                "Expression cron d'ingestion invalide, ingestion non planifiee"
            );
            return;
        }
    };

    if let Err(e) = scheduler.add(job).await {
        tracing::error!(error = %e, "Enregistrement de l'ingestion planifiee impossible");
        return;
    }
    if let Err(e) = scheduler.start().await {
        tracing::error!(error = %e, "Demarrage de l'ordonnanceur d'ingestion impossible");
        return;
    }

    tracing::info!(expression = %expression, "Ingestion planifiee active");

    // L'ordonnanceur doit survivre a cette fonction : `JobScheduler` arrete ses taches quand il est
    // libere. On le garde donc vivant dans une tache detachee plutot que de le laisser tomber au
    // retour, ce qui annulerait silencieusement la planification.
    tokio::spawn(async move {
        let _scheduler = scheduler;
        std::future::pending::<()>().await;
    });
}

/// Expression cron effective. `INGESTION_CRON=off` desactive la planification.
fn cron_expression() -> String {
    std::env::var("INGESTION_CRON")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_CRON.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_expression_par_defaut_a_six_champs() {
        // `croner` attend six champs (secondes incluses), comme Spring. Une expression a cinq
        // champs serait rejetee a l'enregistrement, et l'ingestion ne tournerait jamais — sans
        // autre symptome qu'une ligne de journal au demarrage.
        assert_eq!(DEFAULT_CRON.split_whitespace().count(), 6);
    }

    #[test]
    fn l_expression_par_defaut_declenche_a_deux_heures() {
        let fields: Vec<&str> = DEFAULT_CRON.split_whitespace().collect();
        assert_eq!(fields[0], "0", "secondes");
        assert_eq!(fields[1], "0", "minutes");
        assert_eq!(fields[2], "2", "heures");
    }
}
