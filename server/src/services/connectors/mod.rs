//! Connecteurs vers les sources d'offres d'emploi (ATS et agregateurs).
//!
//! Chaque connecteur implemente [`ats_connector::AtsConnector`] ; le trait porte les comportements
//! communs (parsing d'URL, dates, valeurs par defaut) pour que chaque source ne code que ses
//! specificites.

pub mod ats_connector;
pub mod support;
pub mod boards;
pub mod feeds;
pub mod aggregators;
pub mod scrapers;
pub mod emploi_nc;

pub use ats_connector::*;
pub use support::*;
pub use boards::*;
pub use feeds::*;
pub use aggregators::*;
pub use scrapers::*;
pub use emploi_nc::*;
