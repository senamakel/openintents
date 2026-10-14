# Changelog

All three SDKs share one version. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

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
