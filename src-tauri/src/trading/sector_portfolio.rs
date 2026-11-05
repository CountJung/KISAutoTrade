//! Credential-free sector allocation research. No order manager or broker calls.
mod types;
pub use types::*;
mod planner;
mod validation;
pub use planner::{plan_sector_portfolio, rank_sectors};
mod replay;
pub use replay::preview_sector_portfolio;

pub const SECTOR_ENGINE_VERSION: &str = "sector-portfolio-v1";
pub const SECTOR_SYMBOLS: [&str; 11] = [
    "XLC", "XLY", "XLP", "XLE", "XLF", "XLV", "XLI", "XLB", "XLRE", "XLK", "XLU",
];
pub const ALL_SYMBOLS: [&str; 12] = [
    "XLC", "XLY", "XLP", "XLE", "XLF", "XLV", "XLI", "XLB", "XLRE", "XLK", "XLU", "SPY",
];

#[cfg(test)]
mod tests;
