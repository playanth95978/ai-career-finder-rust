use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Etat d'une partition d'ingestion. Meme vocabulaire que `job_offer.embedding_status`.
pub const INGESTION_STATUS_RUNNING: &str = "RUNNING";
pub const INGESTION_STATUS_COMPLETED: &str = "COMPLETED";
pub const INGESTION_STATUS_FAILED: &str = "FAILED";

/// Une partition d'ingestion journalisee : ce que Spring Batch garderait dans son
/// `StepExecution`.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = crate::db::schema::ingestion_run)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct IngestionRun {
    pub id: Uuid,
    pub run_id: Uuid,
    pub source: String,
    pub partition_key: String,
    pub status: String,
    pub read_count: i32,
    pub written_count: i32,
    pub skipped_count: i32,
    pub error: Option<String>,
    pub started_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::db::schema::ingestion_run)]
pub struct NewIngestionRun {
    pub id: Uuid,
    pub run_id: Uuid,
    pub source: String,
    pub partition_key: String,
    pub status: String,
    pub read_count: i32,
    pub written_count: i32,
    pub skipped_count: i32,
    pub error: Option<String>,
    pub started_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
}

/// Cloture d'une partition : seuls les champs connus a la fin.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::db::schema::ingestion_run)]
pub struct FinishIngestionRun {
    pub status: String,
    pub read_count: i32,
    pub written_count: i32,
    pub skipped_count: i32,
    pub error: Option<String>,
    pub finished_at: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_etats_ne_se_recouvrent_pas() {
        // Les trois constantes servent de discriminant en base : une collision rendrait la reprise
        // incapable de distinguer une partition aboutie d'une partition en echec.
        assert_ne!(INGESTION_STATUS_RUNNING, INGESTION_STATUS_COMPLETED);
        assert_ne!(INGESTION_STATUS_COMPLETED, INGESTION_STATUS_FAILED);
        assert_ne!(INGESTION_STATUS_RUNNING, INGESTION_STATUS_FAILED);
    }

    #[test]
    fn une_partition_en_cours_na_pas_de_date_de_fin() {
        let run = IngestionRun {
            id: Uuid::nil(),
            run_id: Uuid::nil(),
            source: "FRANCE_TRAVAIL".to_string(),
            partition_key: "domaine:M".to_string(),
            status: INGESTION_STATUS_RUNNING.to_string(),
            read_count: 0,
            written_count: 0,
            skipped_count: 0,
            error: None,
            started_at: chrono::NaiveDateTime::UNIX_EPOCH,
            finished_at: None,
        };
        assert!(run.finished_at.is_none());
        assert_eq!(run.clone().partition_key, "domaine:M");
    }
}
