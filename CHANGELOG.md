# Changelog

All three SDKs share one version. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Agent sign-in: when the shop wants the payer signed in and asks for a
  phone number or email and then a one-time code, the agent signs them in
  itself. Each time it needs something only the payer knows, the intent
  waits in the new status `requires_input` with `input_request` (`id`,
  `kind`, `shop`, `prompt`, `channel`, `length`, `url`, `expires_at`,
  `attempts_left`), answered with `POST /intents/{id}/input` (`value`,
  `decline` or `done`; `409 input_not_waiting` once it has moved on), and
  `intent.requires_input` webhooks. `sign_in_agent: false` on create turns
  it off, and `sign_in_hint` passes the payer's phone or email so the agent
  need not ask. Passwords still go through the sign-in link, now the
  fallback. All three SDKs: the status, `input_request`, `sign_in_agent`,
  `sign_in_hint`, the answer (`intents.input` / `answer_intent_input`), and
  `wait` returning at `requires_input` by default (`returnOnInput` /
  `return_on_input`). The CLIs print the prompt and exit `5`, and
  `openintents input` answers.

- Sign-in links: when the shop wants the payer signed in, an intent waits in
  the new status `requires_login` with `sign_in: { url, shop, expires_at }`
  for the payer, instead of failing; `intent.requires_login` webhooks, and
  status reasons `login_required` and `signed_in`. `sign_in_link: false` on
  create keeps the old behaviour (fail with `login_required`). All three
  SDKs: the status, `sign_in`, `sign_in_link`, and `wait` returning at
  `requires_login` by default (`returnOnSignIn` / `return_on_sign_in`). The
  CLIs print the link and exit `4`.

- Spec: browser profiles (`/profiles`), browser sessions (`/sessions`, with
  commands, screenshots and release, and a `sessionEvent` webhook), event
  streams for intents and sessions, and `lease` on the error body for
  `profile_busy`. It also says how sessions behave: why they close
  (`data_limit` among the reasons), when the refs of a snapshot are
  forgotten, which text and keys a command may send, what a session refuses
  (`payment_not_allowed`), every failure code, that at most 2 screenshots
  of a session wait at once (`429 too_many_commands`), and that opening one
  answers `503 sessions_unavailable` while sessions are not open to the key.
- Spec: what the API already serves but the spec left out: customers
  (`/customers`, with the `external_id` filter), top-ups (`/topups`),
  `customer` on intents (and the `customer` filter on `GET /intents`), `held`
  and `customer` on the balance (and `?customer=`), and the server's limits
  and idempotency rules.
- Spec conventions: test and live keys share customers, intents, balances
  and profiles; unknown request fields are refused; new enum values may
  appear.
- SDKs (TypeScript, Python, Rust): customers (create, retrieve, update,
  list), top-ups (create, retrieve), a customer's balance, `customer` when
  creating and listing intents, and the balance's `held` and `customer`.
  Customer and top-up creates send an `Idempotency-Key`, like intents.
- SDKs: browser profiles (list, create, retrieve, forget); browser sessions
  (open, list, retrieve, release, run and retrieve commands, screenshots);
  intent and session events, as a page or as a live stream that resumes
  after a dropped connection (`Last-Event-ID`) and ends when the timeline is
  over (`204`); the error's `lease`; and session webhook events. Session
  creates, releases and commands send an `Idempotency-Key`, so a retry never
  runs a command twice.

### Changed

- TypeScript and Python: `WebhookEvent` is a union of intent and session
  events; check `type` before reading `data`.
- Rust: `WebhookEvent.data` is a `WebhookData` (`Intent`, `Session` or
  `Other`, with `intent()` and `session()` helpers), `Error::Api` has a
  `lease`, and a stream that keeps closing empty ends with
  `Error::StreamClosed`.

## [0.1.0]

### Added

- OpenAPI spec for the v1 preview API (`spec/openapi.yaml`).
- TypeScript SDK and `openintents` CLI (`@tinyhumansai/openintents`).
- Python SDK with sync and async clients (`openintents`).
- Rust SDK with an optional CLI (`openintents`).
- Webhook signature verification in every SDK, with a shared test vector.
- Agent skill and Claude Code plugin.
