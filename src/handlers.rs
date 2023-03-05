use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use ethers::types::H256;
use std::sync::Arc;

use mev_wallet::{MevTxBuilder, SignedMevTx};

use crate::{SearcherService, responses::ApiResponse};

/// Fallback handler that returns a 404 with body `"unknown route"`
pub async fn return_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "unknown route")
}

/// Handler for healthcheck
pub async fn return_200() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn bid_handler<T>(
    State(bidder): State<Arc<T>>,
    Json(mev_tx): Json<MevTxBuilder>,
) -> Response
where
    T: SearcherService + Send + Sync,
{
    let res = bidder.bid(&mev_tx).await;
    match res {
        Ok(resp) => {
            tracing::info!(%resp, "Responding to bid request");
            ApiResponse::new(resp).into_response()
        }
        Err(e) => {
            tracing::error!(err = %e, "error while bidding");
            (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
        }
    }
}

pub async fn bundle_handler<T>(
    auth: Option<Path<H256>>,
    State(bidder): State<Arc<T>>,
    Json(signed_mev_tx): Json<SignedMevTx>,
) -> Response
where
    T: SearcherService + Send + Sync,
{
    let auth = match auth {
        Some(Path(auth)) => Some(auth),
        _ => None,
    };
    let res = bidder.bundle(&signed_mev_tx, auth).await;
    match res {
        Ok(resp) => {
            tracing::info!(%resp, "Responding to bundle request");
            ApiResponse::new(resp).into_response()
        }
        Err(e) => {
            tracing::error!(err = %e, "Error while Bundling");
            (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
        }
    }
}
