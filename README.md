# MevWallet RTB API

This repo provides a standard interface for a real-time bidding API. Searchers
run the API to provide bids to users. The bid represents the Searcher's desired
tip for including that transaction in their current bundle. Users SHOULD accept
one or more bids, and respond with signed MevTx objects to `/bundle`

## Implementing the RTB API

This crate provides a webserver wrapping a `SearcherService`. The service has a
minimal interface, and can encapsulate arbitrary logic.

```rust
pub trait SearcherService {
    async fn bid(&self, tx: &MevTxBuilder) -> eyre::Result<responses::BidResponse>;

    async fn bundle(
        &self,
        tx: &SignedMevTx,
        auth: Option<H256>,
    ) -> eyre::Result<responses::BundleResponse>;
}
```

The `bid` function implements the `/bid` endpoint. It accepts a partial MevTx
and returns a bid, or a rejection. If the user is willing to accept a bid, they
sign the transaction and call the `/bundle` endpoint. Bids are issued against a
specific block, and may include an auth token. This function SHOULD simulate
MEV extraction and respond to the user with a quote for inclusion. This
function MUST respond with a proposed tip, or a rejection.

The `bundle` function implements the `/bundle` endpoint. It accepts a signed
MEV transaction, and appends it to a bundle. The searcher SHOULD re-simulate
the transaction against their current bundle state to ensure it is still
profitable. The Searcher MUST respond with either a status, a rejection, or a
new bid.

Once the trait has been implemented, the library handles spinning up a server
for it. The Searcher SHOULD put this behind a rate limiter, or other reverse
proxy

```rust
#[tokio::main]
fn main() {
    // instantiate your bidding service
    let my_bidder = MyBidder::new();
    // Your bidder, IP and Port
    rtb::serve(my_bidder, ([0, 0, 0, 0], 80)).await
}
```

### Writing a bidding service

Bidding services SHOULD tap into your MEV simulation architecture, either
directly or via IPC. You are free to add arbitrary logic or actions.

A simple flow might be:

- Choose a profit margin %
- Choose a minimum inclusion fee
- Simulate the transaction against your current bundle state
  - Reject reverting transactions
  - Output total extractable MEV
- If MEV is positive:
  - Quote `-1 - (total_extractable - (1 - profit_margin))`
- If MEV is 0:
  - Quote the minimum inclusion fee

### Routes

```
POST /bid
POST /bundle
GET /healthcheck
```

All routes follow the convention `{ "response": ... }`

#### `/bid`

A user posts a partial MEV transaction. If the TX is biddable, the server MUST
respond with a bid. The bid MUST conform to one of the 4 formats below

POST: A partially constructed MEV transaction as JSON
RESPONSES:

```
{
    "response": {
        "accept": {
            "tip": "0x1234", // I256
            "block": "0x1234" // U256
        }
    }
}

{
    "response": {
        "acceptWithAuth": {
            "tip": "0x1234", // I256
            "block": "0x1234", // U256
            "token": "0x1234" // H256
        }
    }
}

{
    "response": "decline"
}

{
    "response": {
        "incomplete": "details of missing info" // String
    }
}
```

#### `/bundle`

POST: A signed, executable MEV transaction as JSON
RESPONSES:

```
{
    "response": "bundled"
}

{
    "response": {
        "tipTooLow": "0x1234" // I256
    }
}

{
    "response": {
        "newBid": {...} // any valid /bid response body
    }
}

{
    "response": "unknownToken"
}

{
    "response": {
        "rejection": "rejection message"
    }
}
```
