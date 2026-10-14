# openintents (Python)

Python SDK for [OpenIntents](https://openintents.io), the agentic API to buy
anything. Send an intent in plain language; the API does the agentic work in a
real browser, gets it paid, and resolves to a receipt. Inference, browser,
compute and bandwidth are free.

> Preview: the API is still being built, so it may change before launch.

```bash
pip install openintents
```

## Usage

```python
from openintents import OpenIntents

oi = OpenIntents()  # reads OPENINTENTS_API_KEY

intent = oi.intents.create(
    "flat white, oat milk, pickup 8:45 at the Blue Bottle on 5th",
    max_amount=1000,  # cents: never spend more than $10.00
)

intent = oi.intents.wait(intent["id"])  # returns when final, or when it needs payment

if intent["status"] == "requires_payment":
    print("Pay here:", intent["payment"]["payment_url"])
elif intent["status"] == "completed":
    receipt = intent["result"]
    print(receipt["merchant"], receipt["total"], receipt["confirmation"])
```

Async:

```python
from openintents import AsyncOpenIntents

async with AsyncOpenIntents() as oi:
    intent = await oi.intents.create("book an Uber to SFO at 7am", max_amount=6000)
    intent = await oi.intents.wait(intent["id"])
```

Also: `oi.intents.get(id)`, `oi.intents.list(status=..., customer=..., limit=...)`,
`oi.intents.list_all()`, `oi.intents.cancel(id)`, `oi.balance.get(customer=...)`,
`oi.customers.create/get/update/list/list_all` and `oi.topups.create/get`.
Pass `customer=` to `intents.create` to buy for one of your customers.
Errors raise `OpenIntentsError` (`status`, `type`, `code`, `param`, and
`lease` with `profile_busy`); 429s, 5xx and network errors are retried with
backoff. Every create (but a profile's), `cancel`, `release` and
`run_command` sends an `Idempotency-Key` automatically.

## Live events

Follow an intent as it runs. The stream resumes by itself after a dropped
connection, and ends once the intent is over:

```python
for event in oi.intents.stream_events(intent["id"]):
    print(event["type"], event["data"])  # status, step, checkout
```

`oi.intents.list_events(id, limit=..., cursor=...)` pages the same timeline.
On `AsyncOpenIntents`, `stream_events` is an async iterator: `async for`.

## Browser sessions

A session opens a payer's browser profile without an intent: to sign them in
to a shop once, so later intents find them signed in. A session never pays
or places an order.

```python
session = oi.sessions.create(customer="cus_4b81d0", start_url="https://shop.example/login")
for event in oi.sessions.stream_events(session["id"]):
    if event["type"] == "status" and event["data"]["status"] == "open":
        break
page = oi.sessions.run_command(session["id"], {"type": "snapshot"})  # page["snapshot"] has the refs
oi.sessions.run_command(session["id"], {"type": "act", "ref": "e12", "action": "type", "text": "ada@example.com"})
jpeg = oi.sessions.screenshot(session["id"])  # bytes
oi.sessions.release(session["id"])  # the profile keeps the sign-in
```

Also `oi.sessions.list/list_all/get/get_command/list_events` and
`oi.profiles.list/list_all/create/get/forget`.

## Webhooks

```python
from openintents import verify_webhook, WebhookVerificationError

event = verify_webhook(request_body_bytes, request.headers.get("OpenIntents-Signature"), "whsec_...")
```

Session events (`session.opened`, `session.closed`, `session.failed`) carry a
session in `data`: check `event["type"]` first.

## Development

```bash
uv run --extra dev pytest
uv run --extra dev ruff check src tests
```

Docs: <https://openintents.io/docs>. License: GPL-3.0-or-later.
