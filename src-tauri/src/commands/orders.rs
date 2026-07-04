use super::*;

// ────────────────────────────────────────────────────────────────────
// 주문
// ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PlaceOrderInput {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: u64,
    pub price: u64,
}

#[tauri::command]
pub async fn place_order(
    input: PlaceOrderInput,
    state: State<'_, AppState>,
) -> CmdResult<OrderResponse> {
    use crate::api::rest::{OrderSide, OrderType};

    let side = match input.side.as_str() {
        "buy" | "Buy" => OrderSide::Buy,
        "sell" | "Sell" => OrderSide::Sell,
        other => {
            return Err(CmdError {
                code: "INVALID_SIDE".into(),
                message: format!("알 수 없는 주문 방향: {}", other),
            })
        }
    };

    let order_type = match input.order_type.as_str() {
        "limit" | "Limit" => OrderType::Limit,
        "market" | "Market" => OrderType::Market,
        other => {
            return Err(CmdError {
                code: "INVALID_ORDER_TYPE".into(),
                message: format!("알 수 없는 주문 유형: {}", other),
            })
        }
    };

    let req = OrderRequest {
        symbol: input.symbol,
        side,
        order_type,
        quantity: input.quantity,
        price: input.price,
    };
    let client = state.rest_client.read().await.clone();
    client.place_order(&req).await.map_err(CmdError::from)
}
