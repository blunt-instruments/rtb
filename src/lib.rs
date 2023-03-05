use std::{net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use ethers::types::H256;
use handlers::{bid_handler, bundle_handler, return_200, return_404};
use mev_wallet::{MevTxBuilder, SignedMevTx};
use tokio::task::JoinHandle;

/// Services Responses
pub mod responses;

/// Service Handlers
pub mod handlers;

#[async_trait::async_trait]
pub trait SearcherService {
    async fn bid(&self, tx: &MevTxBuilder) -> eyre::Result<responses::BidResponse>;

    async fn bundle(
        &self,
        tx: &SignedMevTx,
        auth: Option<H256>,
    ) -> eyre::Result<responses::BundleResponse>;
}

pub fn serve<T>(t: T, socket: impl Into<SocketAddr>) -> JoinHandle<()>
where
    T: SearcherService + Send + Sync + 'static,
{
    let app = Router::new()
        .route("/healthcheck", get(return_200))
        .route("/bid", post(bid_handler))
        .route("/accept/:auth", post(bundle_handler))
        .route("/accept", post(bundle_handler))
        .fallback(return_404)
        .with_state(Arc::new(t));

    let addr = socket.into();
    tokio::spawn(async move {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap()
    })
}
