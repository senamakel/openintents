# openintents (Rust)

Rust SDK for [OpenIntents](https://openintents.io), the agentic API to buy
anything. Async (tokio + reqwest with rustls).

> Preview: the API is still being built, so it may change before launch.

```toml
[dependencies]
openintents = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

```rust
use openintents::{Client, CreateIntent, IntentStatus};

#[tokio::main]
async fn main() -> openintents::Result<()> {
    let oi = Client::from_env()?; // OPENINTENTS_API_KEY
    let intent = oi
        .create_intent(CreateIntent::new("book an Uber to SFO at 7am").max_amount(6000))
        .await?;
    let intent = oi.wait(&intent.id, Default::default()).await?;
    if intent.status == IntentStatus::RequiresPayment {
        println!("pay here: {}", intent.payment.unwrap().payment_url);
    }
    Ok(())
}
```

Also: `get_intent`, `list_intents`, `cancel_intent`, `balance`,
`balance_for`, `create_customer`, `get_customer`, `update_customer`,
`list_customers`, `create_topup`, `get_topup`, and
`webhooks::verify_webhook` for the `OpenIntents-Signature` header (session
events carry a session: see `WebhookData`). `CreateIntent::customer` buys
for one of your customers. Unknown fields and statuses from newer API
versions are tolerated. `Error::Api` has the API's `code`, and the `lease`
with `profile_busy`.

## Live events and browser sessions

`stream_intent_events` follows an intent as it runs: an `EventStream`, read
with `next()` or as a futures `Stream`. It resumes by itself after a dropped
connection, and ends once the intent is over. `list_intent_events` pages
the same timeline.

A session opens a payer's browser profile without an intent: to sign them in
to a shop once, so later intents find them signed in. A session never pays
or places an order.

```rust
use openintents::{Client, CreateSession, SessionCommand};

async fn sign_in(oi: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let session = oi
        .create_session(CreateSession::default().customer("cus_4b81d0").start_url("https://shop.example/login"))
        .await?;
    let mut events = oi.stream_session_events(&session.id, None);
    while let Some(event) = events.next().await {
        let event = event?;
        if event.kind == "status" && event.data["status"] == "open" {
            break;
        }
    }
    let page = oi.run_session_command(&session.id, SessionCommand::snapshot()).await?;
    println!("{}", page.snapshot.unwrap_or_default()); // the refs to act on
    oi.run_session_command(&session.id, SessionCommand::type_text("e12", "ada@example.com"))
        .await?;
    let jpeg: Vec<u8> = oi.session_screenshot(&session.id).await?;
    println!("{} bytes of JPEG", jpeg.len());
    oi.release_session(&session.id).await?; // the profile keeps the sign-in
    Ok(())
}
```

Also `list_sessions`, `get_session`, `get_session_command`,
`list_session_events`, and for profiles `list_profiles`, `create_profile`,
`get_profile` and `forget_profile`.

A CLI binary is included behind a feature:

```bash
cargo install openintents --features cli
openintents run "flat white, oat milk" --max 10 --wait
```

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --features cli -- -D warnings
cargo test
```

Docs: <https://openintents.io/docs>. License: GPL-3.0-or-later.
