use super::{
    http::url_encode,
    support::{new_toss_client_order_id, toss_rate_limit_group, validate_client_order_id},
    types::TossApiResponse,
    *,
};
use crate::broker::{
    BrokerAccountId, BrokerCurrency, BrokerId, BrokerMarket, BrokerMoney, BrokerQuantity,
    BrokerSymbol,
};

#[test]
fn defaults_to_official_base_url() {
    let adapter = TossBrokerAdapter::default();
    assert_eq!(adapter.base_url(), "https://openapi.tossinvest.com");
    assert_eq!(adapter.broker_id(), BrokerId::Toss);
}

#[test]
fn classifies_toss_rate_limit_groups() {
    assert_eq!(toss_rate_limit_group("POST", "/oauth2/token"), "toss:auth");
    assert_eq!(
        toss_rate_limit_group("GET", "/api/v1/holdings?currency=USD"),
        "toss:account"
    );
    assert_eq!(
        toss_rate_limit_group("GET", "/api/v1/orders/order-1"),
        "toss:order_history"
    );
    assert_eq!(
        toss_rate_limit_group("POST", "/api/v1/orders/order-1/cancel"),
        "toss:order"
    );
    assert_eq!(
        toss_rate_limit_group("GET", "/api/v1/prices?symbols=AAPL"),
        "toss:market"
    );
}

#[test]
fn deserializes_accounts() {
    let json = r#"{
          "result": [
            { "accountNo": "12345678901", "accountSeq": 1, "accountType": "BROKERAGE" }
          ]
        }"#;

    let response: TossApiResponse<Vec<TossAccount>> = serde_json::from_str(json).unwrap();
    assert_eq!(response.result[0].account_no, "12345678901");
    assert_eq!(response.result[0].account_seq, 1);
}

#[test]
fn maps_holding_to_broker_domain() {
    let json = r#"{
          "symbol": "AAPL",
          "name": "Apple Inc.",
          "marketCountry": "US",
          "currency": "USD",
          "quantity": "10.5",
          "lastPrice": "178.5",
          "averagePurchasePrice": "155.3",
          "marketValue": {
            "purchaseAmount": "1630.65",
            "amount": "1874.25",
            "amountAfterCost": "1868.25"
          },
          "profitLoss": {
            "amount": "243.60",
            "amountAfterCost": "237.60",
            "rate": "0.1494",
            "rateAfterCost": "0.1457"
          },
          "dailyProfitLoss": {
            "amount": "25",
            "rate": "0.0141"
          },
          "cost": {
            "commission": "6.0",
            "tax": null
          }
        }"#;
    let item: TossHoldingsItem = serde_json::from_str(json).unwrap();
    let account = BrokerAccountId("1".to_string());

    let holding = item.to_broker_holding(Some(&account)).unwrap();

    assert_eq!(holding.broker, BrokerId::Toss);
    assert_eq!(holding.account_id, Some(account));
    assert_eq!(holding.market, BrokerMarket::Us);
    assert_eq!(holding.current_price.currency, BrokerCurrency::Usd);
    assert_eq!(holding.quantity.0, "10.5");
}

#[test]
fn url_encodes_symbol_filter() {
    assert_eq!(url_encode("BRK.B"), "BRK.B");
    assert_eq!(url_encode("ABC DEF"), "ABC%20DEF");
}

#[test]
fn deserializes_order_preflight_responses() {
    let buying_power_json = r#"{
          "result": {
            "currency": "USD",
            "cashBuyingPower": "3500.5"
          }
        }"#;
    let sellable_json = r#"{
          "result": {
            "sellableQuantity": "5.5"
          }
        }"#;
    let commissions_json = r#"{
          "result": [
            {
              "marketCountry": "KR",
              "commissionRate": "0.015",
              "startDate": "2026-01-01",
              "endDate": null
            },
            {
              "marketCountry": "US",
              "commissionRate": "0.1",
              "startDate": null,
              "endDate": "2026-06-30"
            }
          ]
        }"#;

    let buying_power: TossApiResponse<TossBuyingPower> =
        serde_json::from_str(buying_power_json).unwrap();
    let sellable: TossApiResponse<TossSellableQuantity> =
        serde_json::from_str(sellable_json).unwrap();
    let commissions: TossApiResponse<Vec<TossCommission>> =
        serde_json::from_str(commissions_json).unwrap();

    assert_eq!(
        buying_power.result.money().unwrap().currency,
        BrokerCurrency::Usd
    );
    assert_eq!(buying_power.result.cash_buying_power, "3500.5");
    assert_eq!(sellable.result.quantity().0, "5.5");
    assert_eq!(commissions.result.len(), 2);
    assert_eq!(commissions.result[0].market_country, "KR");
}

#[test]
fn validates_order_create_request_shape() {
    let valid = TossOrderCreateRequest {
        client_order_id: Some("order-001_A".to_string()),
        symbol: "005930".to_string(),
        side: "BUY".to_string(),
        order_type: "LIMIT".to_string(),
        time_in_force: Some("DAY".to_string()),
        quantity: Some("10".to_string()),
        price: Some("70000".to_string()),
        order_amount: None,
        confirm_high_value_order: Some(false),
    };
    assert!(valid.validate().is_ok());

    let mut both_quantity_and_amount = valid.clone();
    both_quantity_and_amount.order_amount = Some("100".to_string());
    assert!(both_quantity_and_amount.validate().is_err());

    let mut invalid_client_order_id = valid;
    invalid_client_order_id.client_order_id = Some("bad id with spaces".to_string());
    assert!(invalid_client_order_id.validate().is_err());

    let generated = new_toss_client_order_id();
    assert!(generated.len() <= 36);
    assert!(validate_client_order_id(Some(&generated)).is_ok());
}

#[test]
fn deserializes_order_api_responses() {
    let create_json = r#"{
          "result": {
            "orderId": "ORD_001",
            "clientOrderId": "client-001"
          }
        }"#;
    let list_json = r#"{
          "result": {
            "orders": [
              {
                "orderId": "ORD_001",
                "symbol": "AAPL",
                "side": "BUY",
                "orderType": "LIMIT",
                "timeInForce": "DAY",
                "status": "PARTIAL_FILLED",
                "price": "185.5",
                "quantity": "5",
                "orderAmount": null,
                "currency": "USD",
                "orderedAt": "2026-03-29T10:00:00+09:00",
                "canceledAt": null,
                "execution": {
                  "filledQuantity": "2",
                  "averageFilledPrice": "185.25",
                  "filledAmount": "370.5",
                  "commission": "0.66",
                  "tax": "0",
                  "filledAt": "2026-03-29T10:00:05+09:00",
                  "settlementDate": null
                }
              }
            ],
            "nextCursor": null,
            "hasNext": false
          }
        }"#;
    let operation_json = r#"{
          "result": {
            "orderId": "ORD_002"
          }
        }"#;

    let created: TossApiResponse<TossOrderResponse> = serde_json::from_str(create_json).unwrap();
    let listed: TossApiResponse<TossPaginatedOrderResponse> =
        serde_json::from_str(list_json).unwrap();
    let modified: TossApiResponse<TossOrderOperationResponse> =
        serde_json::from_str(operation_json).unwrap();

    assert_eq!(created.result.order_id, "ORD_001");
    assert_eq!(
        created.result.client_order_id.as_deref(),
        Some("client-001")
    );
    assert_eq!(listed.result.orders[0].status, "PARTIAL_FILLED");
    assert_eq!(listed.result.orders[0].execution.filled_quantity, "2");
    assert!(!listed.result.has_next);
    assert_eq!(modified.result.order_id, "ORD_002");
}

#[test]
fn builds_order_list_query_path() {
    let query = TossOrderListQuery {
        status: TossOrderListStatus::Closed,
        symbol: Some("BRK.B".to_string()),
        from: Some("2026-03-01".to_string()),
        to: Some("2026-03-31".to_string()),
        cursor: Some("next cursor".to_string()),
        limit: Some(100),
    };

    assert!(query.validate().is_ok());
    assert_eq!(
            query.to_path(),
            "/api/v1/orders?status=CLOSED&symbol=BRK.B&from=2026-03-01&to=2026-03-31&cursor=next%20cursor&limit=100"
        );
}

#[test]
fn deserializes_market_data_responses() {
    let prices_json = r#"{
          "result": [
            {
              "symbol": "005930",
              "timestamp": "2026-03-25T09:30:00.123+09:00",
              "lastPrice": "72000",
              "currency": "KRW"
            },
            {
              "symbol": "AAPL",
              "timestamp": null,
              "lastPrice": "178.5",
              "currency": "USD"
            }
          ]
        }"#;
    let orderbook_json = r#"{
          "result": {
            "timestamp": "2026-03-25T09:30:00.123+09:00",
            "currency": "KRW",
            "asks": [{ "price": "72100", "volume": "8500" }],
            "bids": [{ "price": "72000", "volume": "12000" }]
          }
        }"#;
    let trades_json = r#"{
          "result": [
            {
              "price": "72000",
              "volume": "120",
              "timestamp": "2026-03-25T09:30:42.000+09:00",
              "currency": "KRW"
            }
          ]
        }"#;
    let limits_json = r#"{
          "result": {
            "timestamp": "2026-03-25T09:30:00.123+09:00",
            "upperLimitPrice": "93000",
            "lowerLimitPrice": "50400",
            "currency": "KRW"
          }
        }"#;

    let prices: TossApiResponse<Vec<TossPriceResponse>> =
        serde_json::from_str(prices_json).unwrap();
    let orderbook: TossApiResponse<TossOrderbookResponse> =
        serde_json::from_str(orderbook_json).unwrap();
    let trades: TossApiResponse<Vec<TossTrade>> = serde_json::from_str(trades_json).unwrap();
    let limits: TossApiResponse<TossPriceLimitResponse> =
        serde_json::from_str(limits_json).unwrap();

    let quote = prices.result[0].to_broker_price_quote().unwrap();
    assert_eq!(quote.broker, BrokerId::Toss);
    assert_eq!(quote.market, BrokerMarket::Kr);
    assert_eq!(quote.last, BrokerMoney::krw("72000"));
    assert_eq!(
        prices.result[1].to_broker_price_quote().unwrap().market,
        BrokerMarket::Us
    );
    assert_eq!(orderbook.result.asks[0].price, "72100");
    assert_eq!(orderbook.result.bids[0].volume, "12000");
    assert_eq!(trades.result[0].volume, "120");
    assert_eq!(limits.result.upper_limit_price.as_deref(), Some("93000"));
}

#[test]
fn selects_price_response_case_insensitively() {
    let prices = vec![TossPriceResponse {
        symbol: "SOXL".to_string(),
        timestamp: None,
        last_price: "42.15".to_string(),
        currency: "USD".to_string(),
    }];

    let selected = super::adapter::select_price_response(prices, &BrokerSymbol("soxl".to_string()))
        .expect("canonical Toss response symbol should match stored lowercase ticker");

    assert_eq!(selected.symbol, "SOXL");
}

#[test]
fn deserializes_exchange_rate_response() {
    let json = r#"{
          "result": {
            "baseCurrency": "USD",
            "quoteCurrency": "KRW",
            "rate": "1380.5",
            "midRate": "1375",
            "basisPoint": "40",
            "rateChangeType": "UP",
            "validFrom": "2026-03-25T09:30:00+09:00",
            "validUntil": "2026-03-25T09:31:00+09:00"
          }
        }"#;

    let rate: TossApiResponse<TossExchangeRateResponse> = serde_json::from_str(json).unwrap();

    assert_eq!(rate.result.base_currency, "USD");
    assert_eq!(rate.result.quote_currency, "KRW");
    assert_eq!(rate.result.rate_as_f64().unwrap(), 1380.5);
    assert_eq!(rate.result.valid_until, "2026-03-25T09:31:00+09:00");
}

#[test]
fn deserializes_stock_info_and_unknown_warning_codes() {
    let stocks_json = r#"{
          "result": [
            {
              "symbol": "005930",
              "name": "삼성전자",
              "englishName": "SamsungElec",
              "isinCode": "KR7005930003",
              "market": "KOSPI",
              "securityType": "STOCK",
              "isCommonShare": true,
              "status": "ACTIVE",
              "currency": "KRW",
              "listDate": "1975-06-11",
              "delistDate": null,
              "sharesOutstanding": "5919637922",
              "leverageFactor": null,
              "koreanMarketDetail": { "sector": "전기전자" }
            }
          ]
        }"#;
    let warnings_json = r#"{
          "result": [
            {
              "warningType": "INVESTMENT_RISK",
              "exchange": "KRX",
              "startDate": "2026-03-26",
              "endDate": null
            },
            {
              "warningType": "NEW_WARNING_CODE",
              "exchange": null,
              "startDate": null,
              "endDate": null
            }
          ]
        }"#;

    let stocks: TossApiResponse<Vec<TossStockInfo>> = serde_json::from_str(stocks_json).unwrap();
    let warnings: TossApiResponse<Vec<TossStockWarning>> =
        serde_json::from_str(warnings_json).unwrap();

    assert_eq!(stocks.result[0].market, "KOSPI");
    assert_eq!(stocks.result[0].shares_outstanding, "5919637922");
    assert_eq!(warnings.result[0].warning_type, "INVESTMENT_RISK");
    assert!(warnings.result[0].is_blocking_for_buy());
    assert_eq!(warnings.result[1].warning_type, "NEW_WARNING_CODE");
    assert!(!warnings.result[1].is_blocking_for_buy());
}

#[test]
fn deserializes_market_calendar_responses() {
    let kr_json = r#"{
          "result": {
            "today": {
              "date": "2026-03-25",
              "integrated": {
                "preMarket": null,
                "regularMarket": {
                  "startTime": "2026-03-25T09:00:00+09:00",
                  "singlePriceAuctionStartTime": "2026-03-25T15:20:00+09:00",
                  "endTime": "2026-03-25T15:30:00+09:00"
                },
                "afterMarket": null
              }
            },
            "previousBusinessDay": { "date": "2026-03-24", "integrated": null },
            "nextBusinessDay": { "date": "2026-03-26", "integrated": null }
          }
        }"#;
    let us_json = r#"{
          "result": {
            "today": {
              "date": "2026-03-25",
              "dayMarket": null,
              "preMarket": null,
              "regularMarket": {
                "startTime": "2026-03-25T22:30:00+09:00",
                "endTime": "2026-03-26T05:00:00+09:00"
              },
              "afterMarket": null
            },
            "previousBusinessDay": {
              "date": "2026-03-24",
              "dayMarket": null,
              "preMarket": null,
              "regularMarket": null,
              "afterMarket": null
            },
            "nextBusinessDay": {
              "date": "2026-03-26",
              "dayMarket": null,
              "preMarket": null,
              "regularMarket": null,
              "afterMarket": null
            }
          }
        }"#;

    let kr: TossApiResponse<TossKrMarketCalendarResponse> = serde_json::from_str(kr_json).unwrap();
    let us: TossApiResponse<TossUsMarketCalendarResponse> = serde_json::from_str(us_json).unwrap();

    let kr_regular = kr.result.today.integrated.unwrap().regular_market.unwrap();
    let us_regular = us.result.today.regular_market.unwrap();
    assert_eq!(kr_regular.start_time, "2026-03-25T09:00:00+09:00");
    assert_eq!(
        kr_regular.single_price_auction_start_time.as_deref(),
        Some("2026-03-25T15:20:00+09:00")
    );
    assert_eq!(us_regular.end_time, "2026-03-26T05:00:00+09:00");
}

#[test]
fn maps_toss_candle_to_broker_candle() {
    let json = r#"{
          "result": {
            "candles": [
              {
                "timestamp": "2026-03-25T09:00:00+09:00",
                "openPrice": "71600",
                "highPrice": "72300",
                "lowPrice": "71500",
                "closePrice": "72000",
                "volume": "3521000",
                "currency": "KRW"
              }
            ],
            "nextBefore": "2026-03-24T09:00:00+09:00"
          }
        }"#;
    let response: TossApiResponse<TossCandlePageResponse> = serde_json::from_str(json).unwrap();

    let candle = response.result.candles[0]
        .to_broker_candle(&BrokerSymbol("005930".to_string()))
        .unwrap();

    assert_eq!(candle.market, BrokerMarket::Kr);
    assert_eq!(candle.date, "2026-03-25T09:00:00+09:00");
    assert_eq!(candle.open, BrokerMoney::krw("71600"));
    assert_eq!(candle.close, BrokerMoney::krw("72000"));
    assert_eq!(candle.volume, BrokerQuantity("3521000".to_string()));
    assert_eq!(
        response.result.next_before.as_deref(),
        Some("2026-03-24T09:00:00+09:00")
    );
}

#[tokio::test]
async fn validates_market_data_query_limits_before_request() {
    let client = TossOpenApiClient::without_credentials("https://example.invalid");
    let too_many = (0..201)
        .map(|i| BrokerSymbol(format!("SYM{i}")))
        .collect::<Vec<_>>();

    assert!(client.list_prices(&[]).await.is_err());
    assert!(client.list_prices(&too_many).await.is_err());
    assert!(client.list_stocks(&[]).await.is_err());
    assert!(client.list_stocks(&too_many).await.is_err());
    assert!(client
        .list_warnings(&BrokerSymbol("bad symbol".to_string()))
        .await
        .is_err());
    assert!(client
        .get_kr_market_calendar(Some("20260325"))
        .await
        .is_err());
    assert!(client
        .get_us_market_calendar(Some("2026/03/25"))
        .await
        .is_err());
    assert!(client
        .list_trades(&BrokerSymbol("AAPL".to_string()), Some(51))
        .await
        .is_err());
    assert!(client
        .get_candles(
            &BrokerSymbol("AAPL".to_string()),
            "1h",
            Some(100),
            None,
            None
        )
        .await
        .is_err());
    assert!(client
        .get_candles(
            &BrokerSymbol("AAPL".to_string()),
            "1d",
            Some(201),
            None,
            None
        )
        .await
        .is_err());
}
