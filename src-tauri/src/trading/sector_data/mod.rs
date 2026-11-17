//! Quote-only preparation boundary. Validation never authenticates or calls a provider.
mod collector;
mod types;
mod validation;
pub use collector::{collect_sector_bundle, SectorQuoteSource};
pub use types::*;
pub use validation::{validate_manifest, validate_sector_bundle};

#[cfg(test)]
pub(crate) mod tests;
