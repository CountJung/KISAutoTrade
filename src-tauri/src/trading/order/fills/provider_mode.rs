//! Persisted KIS order provenance must match the currently selected investment mode.

use super::*;

pub(super) fn validate_pending_kis_mode(
    pending: &super::super::PendingOrder,
    is_paper: bool,
) -> Result<()> {
    if pending_order_provider(pending) != BrokerId::Kis {
        return Ok(());
    }
    let pending_is_paper = match pending.record.provider_tr_id.as_deref() {
        Some("VTTC0802U" | "VTTC0801U" | "VTTT1002U" | "VTTT1006U") => true,
        Some("TTTC0802U" | "TTTC0801U" | "TTTT1002U" | "TTTT1006U") => false,
        _ => anyhow::bail!(
            "KIS pending 주문의 저장된 TR-ID로 투자 모드를 확인할 수 없어 체결·예산 반영을 보류합니다."
        ),
    };
    anyhow::ensure!(
        pending_is_paper == is_paper,
        "KIS pending 주문과 현재 client의 투자 모드가 달라 체결·예산 반영을 보류합니다."
    );
    Ok(())
}

impl OrderManager {
    pub(super) async fn validate_pending_fill_mode(
        &self,
        pending: &super::super::PendingOrder,
    ) -> Result<()> {
        if pending_order_provider(pending) != BrokerId::Kis {
            return Ok(());
        }
        let client = if let Some(context) = self.execution_run_context() {
            self.execution_run_snapshots(&context)
                .ok_or_else(|| anyhow::anyhow!("체결 조회 실행 client를 확인할 수 없습니다."))?
                .0
        } else {
            self.rest_client.read().await.clone()
        };
        validate_pending_kis_mode(pending, client.is_paper())
    }
}
