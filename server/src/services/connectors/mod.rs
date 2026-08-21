//! Connecteurs vers les sources d'offres d'emploi (ATS et agregateurs).
//!
//! Chaque connecteur implemente [`ats_connector::AtsConnector`] ; le trait porte les comportements
//! communs (parsing d'URL, dates, valeurs par defaut) pour que chaque source ne code que ses
//! specificites.

pub mod ats_connector;
pub mod emploi_nc;

pub use ats_connector::*;
pub use emploi_nc::*;
