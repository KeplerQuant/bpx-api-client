mod common;

use bpx_api_client::{
    BpxClient,
    types::{
        history::SortDirection,
        order::{OrderHistory, OrderHistoryParams, OrderStatus, OrderType, Side},
    },
};
use rust_decimal_macros::dec;
use std::collections::BTreeMap;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn get_historical_orders_signs_and_deserializes_response() {
    let mock_server = MockServer::start().await;
    let response = r#"[{
        "id":"123",
        "createdAt":"2026-10-06T01:02:03.456789",
        "executedQuantity":"1.5",
        "executedQuoteQuantity":"150",
        "expiryReason":null,
        "orderType":"Limit",
        "postOnly":true,
        "price":"100",
        "quantity":"2",
        "quoteQuantity":null,
        "selfTradePrevention":"RejectTaker",
        "status":"PartiallyFilled",
        "side":"Bid",
        "stopLossTriggerPrice":null,
        "stopLossLimitPrice":null,
        "stopLossTriggerBy":null,
        "symbol":"SOL_USDC",
        "takeProfitTriggerPrice":null,
        "takeProfitLimitPrice":null,
        "takeProfitTriggerBy":null,
        "timeInForce":"GTC",
        "triggerBy":null,
        "triggerPrice":null,
        "triggerQuantity":null,
        "clientId":42,
        "systemOrderType":null,
        "strategyId":null,
        "slippageTolerance":null,
        "slippageToleranceType":null
    }]"#;

    Mock::given(method("GET"))
        .and(path("/wapi/v1/history/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(response, "application/json"))
        .mount(&mock_server)
        .await;

    let client = BpxClient::builder()
        .base_url(mock_server.uri())
        .secret(common::test_secret())
        .build()
        .expect("client should build");
    let params = OrderHistoryParams::default()
        .with_order_id("123")
        .with_symbol("SOL_USDC")
        .with_market_type("SPOT")
        .with_limit(100)
        .with_offset(0)
        .with_sort_direction(SortDirection::Desc);

    let orders = client
        .get_historical_orders(params)
        .await
        .expect("request should succeed");

    assert_eq!(
        orders,
        vec![OrderHistory {
            id: "123".to_string(),
            created_at: "2026-10-06T01:02:03.456789".to_string(),
            executed_quantity: dec!(1.5),
            executed_quote_quantity: dec!(150),
            expiry_reason: None,
            order_type: OrderType::Limit,
            post_only: Some(true),
            price: Some(dec!(100)),
            quantity: Some(dec!(2)),
            quote_quantity: None,
            self_trade_prevention: Some(
                bpx_api_client::types::order::SelfTradePrevention::RejectTaker
            ),
            status: OrderStatus::PartiallyFilled,
            side: Side::Bid,
            stop_loss_trigger_price: None,
            stop_loss_limit_price: None,
            stop_loss_trigger_by: None,
            symbol: "SOL_USDC".to_string(),
            take_profit_trigger_price: None,
            take_profit_limit_price: None,
            take_profit_trigger_by: None,
            time_in_force: Some(bpx_api_client::types::order::TimeInForce::GTC),
            trigger_by: None,
            trigger_price: None,
            trigger_quantity: None,
            client_id: Some(42),
            system_order_type: None,
            strategy_id: None,
            slippage_tolerance: None,
            slippage_tolerance_type: None,
        }]
    );

    let requests = mock_server
        .received_requests()
        .await
        .expect("wiremock should record requests");
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert!(request.headers.contains_key("x-signature"));

    let pairs: BTreeMap<_, _> = request
        .url
        .query()
        .expect("order history request should include query parameters")
        .split('&')
        .map(|segment| segment.split_once('=').expect("bare query key"))
        .collect();
    assert_eq!(
        pairs,
        BTreeMap::from([
            ("limit", "100"),
            ("marketType", "SPOT"),
            ("offset", "0"),
            ("orderId", "123"),
            ("sortDirection", "Desc"),
            ("symbol", "SOL_USDC"),
        ])
    );
}
