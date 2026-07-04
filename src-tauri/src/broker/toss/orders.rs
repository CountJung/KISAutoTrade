use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use super::{
    http::url_encode,
    support::{
        new_toss_client_order_id, validate_client_order_id, validate_iso_date,
        validate_optional_decimal, validate_order_side, validate_order_type,
        validate_time_in_force, validate_toss_symbol,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TossOrderListStatus {
    Open,
    Closed,
}

impl TossOrderListStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Closed => "CLOSED",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TossOrderListQuery {
    pub status: TossOrderListStatus,
    pub symbol: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u8>,
}

impl TossOrderListQuery {
    pub fn open() -> Self {
        Self {
            status: TossOrderListStatus::Open,
            symbol: None,
            from: None,
            to: None,
            cursor: None,
            limit: None,
        }
    }

    pub(super) fn validate(&self) -> anyhow::Result<()> {
        if let Some(symbol) = &self.symbol {
            validate_toss_symbol(symbol)?;
        }
        if let Some(from) = &self.from {
            validate_iso_date(from)?;
        }
        if let Some(to) = &self.to {
            validate_iso_date(to)?;
        }
        if let Some(limit) = self.limit {
            if !(1..=100).contains(&limit) {
                return Err(anyhow!(
                    "토스증권 주문 목록 limit은 1~100 범위여야 합니다: {limit}"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn to_path(&self) -> String {
        let mut params = vec![format!("status={}", self.status.as_str())];
        if let Some(symbol) = &self.symbol {
            params.push(format!("symbol={}", url_encode(symbol)));
        }
        if let Some(from) = &self.from {
            params.push(format!("from={}", url_encode(from)));
        }
        if let Some(to) = &self.to {
            params.push(format!("to={}", url_encode(to)));
        }
        if let Some(cursor) = &self.cursor {
            params.push(format!("cursor={}", url_encode(cursor)));
        }
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        format!("/api/v1/orders?{}", params.join("&"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderCreateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm_high_value_order: Option<bool>,
}

impl TossOrderCreateRequest {
    pub fn with_generated_client_order_id(mut self) -> Self {
        self.client_order_id = Some(new_toss_client_order_id());
        self
    }

    pub(super) fn validate(&self) -> anyhow::Result<()> {
        validate_client_order_id(self.client_order_id.as_deref())?;
        validate_toss_symbol(&self.symbol)?;
        validate_order_side(&self.side)?;
        validate_order_type(&self.order_type)?;
        if let Some(time_in_force) = &self.time_in_force {
            validate_time_in_force(time_in_force)?;
        }
        validate_optional_decimal("quantity", self.quantity.as_deref())?;
        validate_optional_decimal("price", self.price.as_deref())?;
        validate_optional_decimal("orderAmount", self.order_amount.as_deref())?;

        match (self.quantity.is_some(), self.order_amount.is_some()) {
            (true, false) | (false, true) => Ok(()),
            _ => Err(anyhow!(
                "토스증권 주문 생성은 quantity 또는 orderAmount 중 정확히 하나만 허용합니다"
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderModifyRequest {
    pub order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm_high_value_order: Option<bool>,
}

impl TossOrderModifyRequest {
    pub(super) fn validate(&self) -> anyhow::Result<()> {
        validate_order_type(&self.order_type)?;
        validate_optional_decimal("quantity", self.quantity.as_deref())?;
        validate_optional_decimal("price", self.price.as_deref())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderResponse {
    pub order_id: String,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderOperationResponse {
    pub order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossPaginatedOrderResponse {
    pub orders: Vec<TossOrder>,
    pub next_cursor: Option<String>,
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub time_in_force: String,
    pub status: String,
    pub price: Option<String>,
    pub quantity: String,
    pub order_amount: Option<String>,
    pub currency: String,
    pub ordered_at: String,
    pub canceled_at: Option<String>,
    pub execution: TossOrderExecution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossOrderExecution {
    pub filled_quantity: String,
    pub average_filled_price: Option<String>,
    pub filled_amount: Option<String>,
    pub commission: Option<String>,
    pub tax: Option<String>,
    pub filled_at: Option<String>,
    pub settlement_date: Option<String>,
}
