use serde::{ Deserialize, Serialize };
use std::cmp::{ PartialEq };
use strum_macros::{AsRefStr, IntoStaticStr};
use crate::ids::{ConnectionID, MarketID};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMessage {
	pub id: Option<u32>,
    pub connection_id: ConnectionID,
    pub connections_available: Option<u16>,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum StatusCode {
    #[serde(rename = "SUCCESS")]
    Success,
    #[serde(rename = "FAILURE")]
    Failure,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusMessage  {
	pub id: Option<u32>,
    pub status_code: StatusCode,
	pub error_code: Option<StreamErrorCode>,
	pub connection_id: Option<ConnectionID>,
	pub error_message: Option<String>,
	pub connection_closed: Option<bool>,
    pub connections_available: Option<u16>,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize, AsRefStr, IntoStaticStr,)]
pub enum StreamErrorCode {
    #[strum(serialize = "INVALID_INPUT")]
    #[serde(rename = "INVALID_INPUT")]
    InvalidInput, // Failure code returned when an invalid input is provided (could not deserialize the message)
    #[strum(serialize = "TIMEOUT")]
    #[serde(rename = "TIMEOUT")]
    Timeout, // Failure code when a client times out (i.e. too slow sending data)
    #[strum(serialize = "NO_APP_KEY")]
    #[serde(rename = "NO_APP_KEY")]
    NoAppKey, // Failure code returned when an application key is not found in the message
    #[strum(serialize = "INVALID_APP_KEY")]
    #[serde(rename = "INVALID_APP_KEY")]
    InvalidAppKey, // Failure code returned when an invalid application key is received
    #[strum(serialize = "NO_SESSION")]
    #[serde(rename = "NO_SESSION")]
    NoSession, // Failure code returned when a session token is not found in the message
    #[strum(serialize = "INVALID_SESSION_INFORMATION")]
    #[serde(rename = "INVALID_SESSION_INFORMATION")]
    InvalidSessionInfo, // Failure code returned when an invalid session token is received
    #[strum(serialize = "NOT_AUTHORIZED")]
    #[serde(rename = "NOT_AUTHORIZED")]
    NotAuthorized, // Failure code returned when client is not authorized to perform the operation
    #[strum(serialize = "MAX_CONNECTION_LIMIT_EXCEEDED")]
    #[serde(rename = "MAX_CONNECTION_LIMIT_EXCEEDED")]
    MaxConnectionLimitExceeded, // Failure code returned when a client tries to create more connections than allowed to
    #[strum(serialize = "TOO_MANY_REQUESTS")]
    #[serde(rename = "TOO_MANY_REQUESTS")]
    TooManyRequests, // Failure code returned when a client makes too many requests within a short time period
    #[strum(serialize = "SUBSCRIPTION_LIMIT_EXCEEDED")]
    #[serde(rename = "SUBSCRIPTION_LIMIT_EXCEEDED")]
    SubscriptionLimitExceeded, // Customer tried to subscribe to more markets than allowed to - set to 200 markets by default
    #[strum(serialize = "INVALID_CLOCK")]
    #[serde(rename = "INVALID_CLOCK")]
    InvalidClock, // Failure code returned when an invalid clock is provided on re-subscription (check initialClk / clk supplied)
    #[strum(serialize = "UNEXPECTED_ERROR")]
    #[serde(rename = "UNEXPECTED_ERROR")]
    UnexpectedError, // Failure code returned when an internal error occurred on the server
    #[strum(serialize = "CONNECTION_FAILED")]
    #[serde(rename = "CONNECTION_FAILED")]
    ConnectionFailed, // Failure code used when the client / server connection is terminated
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "op")]
pub enum Request<'a> {
    #[serde(rename = "authentication")]
    Authentication(AuthMessage<'a>),
    #[serde(borrow)]
    #[serde(rename = "marketSubscription")]
    MarketSub(SubMessage<'a, MarketSub<'a>>),
    #[serde(rename = "orderSubscription")]
    OrderSub(SubMessage<'a, OrderSub<'a>>),
    #[serde(rename = "heartbeat")]
    Heartbeat(HeartbeatMessage),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMessage<'a> {
	pub id: Option<u32>,    
	pub session: &'a str, 
	pub app_key: &'a str,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubMessage<'a, T: Subscription> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(borrow)]
    #[serde(skip_serializing_if = "Option::is_none")]
	pub initial_clk: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clk: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segmentation_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_ms:  Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflate_ms:  Option<u32>,
    
    #[serde(flatten)]
    pub filter: T,
}
pub trait Subscription {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSub<'a> {
    #[serde(borrow)]
	pub market_filter: MarketStreamFilter<'a>,
    pub market_data_filter: MarketDataFilter,
}
impl<'a> Subscription for MarketSub<'a> {}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderSub<'a> {
    #[serde(borrow)]
    pub order_filter: OrderFilter<'a> ,
}
impl<'a>  Subscription for OrderSub<'a>  {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDataFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ladder_levels: Option<u32>,     
	pub fields: Vec<DataFilterField>,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum DataFilterField {
    #[serde(rename = "EX_BEST_OFFERS_DISP")]
    ExBestOffersDisp,
    #[serde(rename = "EX_BEST_OFFERS")]
    ExBestOffers,
    #[serde(rename = "EX_ALL_OFFERS")]
    ExAllOffers,
    #[serde(rename = "EX_TRADED")]
    ExTraded,
    #[serde(rename = "EX_TRADED_VOL")]
    ExTradedVol,
    #[serde(rename = "EX_LTP")]
    ExLtp,
    #[serde(rename = "EX_MARKET_DEF")]
    ExMarketDef,
    #[serde(rename = "SP_TRADED")]
    SpTraded,
    #[serde(rename = "SP_PROJECTED")]
    SpProjected,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketStreamFilter<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ids: Option<Vec<MarketID>>,
    #[serde(borrow)]
    #[serde(skip_serializing_if = "Option::is_none")]
	pub country_codes: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub betting_types: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_types: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub venues: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_type_ids: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_ids: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_in_play_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bsp_market: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFilter<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
	pub customer_strategy_refs: Option<&'a [&'a str]>,
	pub include_overall_position: bool,
	pub partition_matched_by_strategy_ref: bool,
}