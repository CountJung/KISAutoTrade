//! Current-backend persistence boundary for confirmed ownership.
//!
//! Read the primary document directly. A stale JSON backup must never resurrect
//! an earlier position quantity or reset an already-applied fill watermark.
use super::OwnershipLedger;
use crate::trading::order::OrderManager;
use anyhow::Result;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OwnershipManifest {
    version: u32,
    generation: String,
}

impl super::OwnershipIdentity {
    pub(crate) fn capture(
        scope: &crate::broker::BrokerScope,
        profile: Option<&crate::config::AccountProfile>,
        client: &crate::api::rest::KisRestClient,
    ) -> Result<Self> {
        crate::trading::order::execution_run::validate_snapshot(
            scope,
            profile.map(|p| p.id.as_str()),
            client,
            profile,
        )?;
        let mut scope = scope.clone();
        if scope.account_id.is_none() && scope.broker_id == crate::broker::BrokerId::Kis {
            anyhow::ensure!(
                !client.account_no().trim().is_empty(),
                "소유 계좌를 확인할 수 없습니다."
            );
            scope.account_id = Some(crate::broker::BrokerAccountId(
                client.account_no().to_string(),
            ));
        }
        Ok(Self {
            scope,
            profile_id: profile.map(|p| p.id.clone()),
            is_paper: profile.map_or_else(|| client.is_paper(), |p| p.is_paper_trading),
        })
    }
}

impl OrderManager {
    pub(crate) async fn load_ownership_ledger(&mut self) -> Result<OwnershipLedger> {
        let loaded: Result<(OwnershipLedger, bool, bool)> = async {
            let manifest = crate::storage::database::read_managed_json(
                &self.pending_order_store.ownership_manifest_path(),
            )
            .await?
            .map(|content| serde_json::from_str::<OwnershipManifest>(&content))
            .transpose()?;
            if let Some(manifest) = &manifest {
                anyhow::ensure!(
                    manifest.version == 1
                        && uuid::Uuid::parse_str(&manifest.generation).is_ok_and(|id| !id.is_nil()),
                    "체결 소유 원장 식별자가 손상되었습니다."
                );
            }
            let (ledger, missing) = match crate::storage::database::read_managed_json(
                &self.pending_order_store.ownership_path(),
            )
            .await?
            {
                Some(content) => (serde_json::from_str::<OwnershipLedger>(&content)?, false),
                None => {
                    anyhow::ensure!(manifest.is_none(), "체결 소유 원장 원본이 유실되었습니다.");
                    (OwnershipLedger::default(), true)
                }
            };
            if let Some(manifest) = &manifest {
                anyhow::ensure!(
                    manifest.generation == ledger.generation().to_string(),
                    "체결 소유 원장과 영속 식별자의 세대가 다릅니다."
                );
            }
            Ok((ledger, missing, manifest.is_none()))
        }
        .await;
        match loaded {
            Ok((ledger, missing, missing_manifest)) => {
                let generation = ledger.generation().to_string();
                if self
                    .ownership_document_generation
                    .as_ref()
                    .is_some_and(|previous| previous != &generation)
                {
                    let error = anyhow::anyhow!(
                        "체결 소유 원장의 세대가 변경되거나 원본이 유실되었습니다."
                    );
                    self.block_for_persistence_failure(error.to_string());
                    return Err(error);
                }
                if missing {
                    self.save_ownership_ledger(&ledger).await?;
                }
                if missing_manifest {
                    let manifest = OwnershipManifest {
                        version: 1,
                        generation: generation.clone(),
                    };
                    if let Err(error) = crate::storage::write_json(
                        &self.pending_order_store.ownership_manifest_path(),
                        &manifest,
                    )
                    .await
                    {
                        self.block_for_persistence_failure(format!(
                            "체결 소유 원장 식별자 저장 실패: {error}"
                        ));
                        return Err(error);
                    }
                }
                self.ownership_document_generation = Some(generation);
                Ok(ledger)
            }
            Err(error) => {
                self.block_for_persistence_failure(format!(
                    "체결 소유 원장을 읽지 못했습니다: {error}"
                ));
                Err(error)
            }
        }
    }

    /// The caller keeps the old runtime ledger until this exact candidate saves.
    pub(crate) async fn save_ownership_ledger(&mut self, ledger: &OwnershipLedger) -> Result<()> {
        if let Err(error) =
            crate::storage::write_json(&self.pending_order_store.ownership_path(), ledger).await
        {
            self.block_for_persistence_failure(format!(
                "체결 소유 원장을 저장하지 못했습니다: {error}"
            ));
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
