use ethers::types::{H256, I256, U256};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    response: T,
}

impl<T> ApiResponse<T> {
    pub fn new(response: T) -> Self {
        Self { response }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, serde_json::to_string(&self).unwrap()).into_response()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BundleResponse {
    // Added to searcher bundle. Searcher commits to include
    Bundled,
    // Signed tip is lower than bid
    TipTooLow(I256),
    // Searcher wants to adjust the bid
    NewBid(BidResponse),
    // Auth token was bad
    UnknownToken,
    // Other error
    Rejection(String),
}

impl std::fmt::Display for BundleResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleResponse::Bundled => write!(f, "Bundled"),
            BundleResponse::TipTooLow(req) => write!(f, "TipTooLow {req}"),
            BundleResponse::NewBid(resp) => write!(f, "New Quote {resp}"),
            BundleResponse::UnknownToken => write!(f, "UnknownToken"),
            BundleResponse::Rejection(s) => write!(f, "Rejection: {s}"),
        }
    }
}

impl IntoResponse for BundleResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, serde_json::to_string(&self).unwrap()).into_response()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BidResponse {
    Accept { tip: I256, block: U256 },
    AcceptWithAuth { tip: I256, token: H256, block: U256 },
    Decline,
    Incomplete(String),
}

impl std::fmt::Display for BidResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BidResponse::Accept { tip, block } => write!(f, "Accept {tip} @ height {block}"),
            BidResponse::AcceptWithAuth { tip, token, block } => {
                write!(f, "Accept {tip} @ height {block} with {token:?}")
            }
            BidResponse::Decline => write!(f, "Decline"),
            BidResponse::Incomplete(reason) => write!(f, "Incomplete: {reason}"),
        }
    }
}

impl IntoResponse for BidResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, serde_json::to_string(&self).unwrap()).into_response()
    }
}
