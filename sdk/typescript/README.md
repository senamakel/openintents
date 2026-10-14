# @tinyhumansai/openintents

TypeScript SDK and CLI for [OpenIntents](https://openintents.io), the agentic
API to buy anything. Send an intent in plain language; the API does the
agentic work in a real browser, gets it paid, and resolves to a receipt.
Inference, browser, compute and bandwidth are free.

> Preview: the API is still being built, so it may change before launch.

```bash
npm install @tinyhumansai/openintents
```

Zero runtime dependencies. Works in Node 20.3+, Bun, Deno and edge runtimes
(uses `fetch` and Web Crypto).

## SDK

```ts
import { OpenIntents } from "@tinyhumansai/openintents";

const oi = new OpenIntents(); // reads OPENINTENTS_API_KEY

const created = await oi.intents.create({
  intent: "flat white, oat milk, pickup 8:45 at the Blue Bottle on 5th",
  max_amount: 1000, // cents: never spend more than $10.00
});

const intent = await oi.intents.wait(created.id); // final, or needs payment

if (intent.status === "requires_payment") {
  console.log("Pay here:", intent.payment?.payment_url);
} else if (intent.status === "completed") {
  console.log(intent.result?.merchant, intent.result?.total);
}
```

Also: `oi.intents.get(id)`, `oi.intents.list({ status, customer, limit })`,
`for await (const i of oi.intents.listAll())`, `oi.intents.cancel(id)`,
`oi.balance.get({ customer })`, `oi.customers.create/get/update/list/listAll`
and `oi.topups.create/get`. Pass `customer` to `intents.create` to buy for one
of your customers. Errors throw `OpenIntentsError` (`status`, `type`, `code`,
`param`, and `lease` with `profile_busy`). Every create (but a profile's),
`cancel`, `release` and `runCommand` sends an `Idempotency-Key`
automatically; 429s, 5xx and network errors are retried with backoff.

### Live events

Follow an intent as it runs. The stream resumes by itself after a dropped
connection, and ends once the intent is over:

```ts
for await (const event of oi.intents.streamEvents(created.id)) {
  console.log(event.type, event.data); // status, step, checkout
}
```

`oi.intents.listEvents(id, { limit, cursor })` pages the same timeline.

### Browser sessions

A session opens a payer's browser profile without an intent: to sign them in
to a shop once, so later intents find them signed in. A session never pays
or places an order.

```ts
const session = await oi.sessions.create({ customer: "cus_4b81d0", start_url: "https://shop.example/login" });
for await (const event of oi.sessions.streamEvents(session.id)) {
  if (event.type === "status" && event.data.status === "open") break;
}
const page = await oi.sessions.runCommand(session.id, { type: "snapshot" }); // page.snapshot has the refs
await oi.sessions.runCommand(session.id, { type: "act", ref: "e12", action: "type", text: "ada@example.com" });
const jpeg = await oi.sessions.screenshot(session.id); // Uint8Array
await oi.sessions.release(session.id); // the profile keeps the sign-in
```

Also `oi.sessions.list/listAll/get/getCommand/listEvents` and
`oi.profiles.list/listAll/create/get/forget`.

### Webhooks

```ts
import { verifyWebhook } from "@tinyhumansai/openintents";

// Use the raw body, before JSON parsing.
const event = await verifyWebhook(rawBody, req.headers.get("OpenIntents-Signature"), process.env.WEBHOOK_SECRET!);
if (event.type === "intent.requires_payment") notify(event.data.payment?.payment_url);
```

Session events (`session.opened`, `session.closed`, `session.failed`) carry
a session in `data`: check `event.type` first.

## CLI

```bash
npm install -g @tinyhumansai/openintents
openintents login
openintents run "UberX to SFO, leaving 7am" --max 60 --wait
openintents status <id> --wait --json
openintents pay <id>      # open the payment link
openintents balance
```

Exit codes: `0` completed, `1` failed or error, `2` needs payment, `3`
cancelled.

## Development

```bash
pnpm install
pnpm --filter @tinyhumansai/openintents lint
pnpm --filter @tinyhumansai/openintents test
pnpm --filter @tinyhumansai/openintents build
```

Docs: <https://openintents.io/docs>. License: GPL-3.0-or-later.
