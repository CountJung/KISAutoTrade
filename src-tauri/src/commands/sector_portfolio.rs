//! Supplied-data research only. This boundary has no AppState or broker capability.
use super::{CmdError, CmdResult};
use crate::trading::sector_portfolio::{SectorPortfolioInput, SectorPortfolioReport};

pub async fn calculate_sector_portfolio(
    input: SectorPortfolioInput,
) -> CmdResult<SectorPortfolioReport> {
    tokio::task::spawn_blocking(move || {
        crate::trading::sector_portfolio::preview_sector_portfolio(&input).map_err(|message| {
            CmdError {
                code: "INVALID_SECTOR_PORTFOLIO_INPUT".into(),
                message,
            }
        })
    })
    .await
    .map_err(|_| CmdError {
        code: "SECTOR_PORTFOLIO_CALCULATION_FAILED".into(),
        message: "섹터 포트폴리오 계산 작업이 완료되지 않았습니다.".into(),
    })?
}

#[tauri::command]
pub async fn preview_sector_portfolio(
    input: SectorPortfolioInput,
) -> CmdResult<SectorPortfolioReport> {
    calculate_sector_portfolio(input).await
}
