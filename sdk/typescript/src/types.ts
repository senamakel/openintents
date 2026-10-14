/** Types for the OpenIntents v1 API. Source of truth: spec/openapi.yaml. */

export type IntentStatus =
  | "queued"
  | "running"
  | "requires_payment"
  | "completed"
  | "failed"
  | "cancelled";

/** Statuses an intent never leaves. */
export const FINAL_STATUSES: readonly IntentStatus[] = ["completed", "failed", "cancelled"];

export interface CreateIntentParams {
  /** What to buy or book, in natural language. */
  intent: string;
  /** Spending cap in cents; the intent fails rather than exceed it. */
  max_amount?: number;
  /** ISO currency code, lowercase. */
  currency?: string;
  /** Structured details: addresses, names, loyalty numbers. */
  context?: Record<string, unknown>;
  /** Always return a payment link instead of paying from the balance. */
  require_payment_link?: boolean;
  /** Up to 20 of your own key/value pairs. */
  metadata?: Record<string, string>;
  /** A customer of your account to buy for: their balance and browser profile. */
  customer?: string;
}

export interface PaymentRequest {
  reason: "insufficient_balance" | "over_limit" | "requested";
  payment_url: string;
  merchant?: string | null;
  total: number;
  fee: number;
  currency: string;
  expires_at: string;
}

export interface ReceiptItem {
  name: string;
  quantity: number;
  amount: number;
}

export interface Receipt {
  merchant: string;
  items?: ReceiptItem[];
  total: number;
  fee: number;
  currency: string;
  confirmation?: string | null;
  receipt_url?: string | null;
}

export interface IntentError {
  code: string;
  message: string;
}

export interface Intent {
  id: string;
  object: "intent";
  status: IntentStatus;
  intent: string;
  /** The customer it buys for; `null` for your account's own intents. */
  customer?: string | null;
  max_amount?: number | null;
  currency?: string | null;
  context?: Record<string, unknown> | null;
  metadata?: Record<string, string> | null;
  /** Set when `status` is `requires_payment`. */
  payment?: PaymentRequest | null;
  /** Set when `status` is `completed`. */
  result?: Receipt | null;
  /** Set when `status` is `failed`. */
  error?: IntentError | null;
  test: boolean;
  created_at: string;
  completed_at?: string | null;
}

export interface ListIntentsParams {
  status?: IntentStatus;
  /** Only intents bought for this customer. */
  customer?: string;
  /** 1 to 100, default 20. */
  limit?: number;
  cursor?: string;
}

export interface IntentList {
  object: "list";
  data: Intent[];
  next_cursor?: string | null;
}

export interface Balance {
  object: "balance";
  /** What intents can pay from now. */
  available: number;
  /** Reserved for intents at their payment step. */
  held?: number;
  currency: string;
  /** The customer whose balance this is; `null` for your account's. */
  customer?: string | null;
  monthly_limit?: number | null;
  spent_this_month?: number;
}

export interface BalanceParams {
  /** A customer's balance instead of your account's. */
  customer?: string;
}

/** Someone you buy for, with their own balance and browser profile. */
export interface Customer {
  id: string;
  object: "customer";
  /** Your own ID for them, unique in your account. */
  external_id?: string | null;
  name?: string | null;
  email?: string | null;
  metadata?: Record<string, string>;
  created_at: string;
}

export interface CustomerParams {
  /** Unique in your account; one already in use is a 409 conflict. */
  external_id?: string;
  name?: string;
  email?: string;
  /** Up to 20 pairs. On update it replaces the whole map. */
  metadata?: Record<string, string>;
}

export interface ListCustomersParams {
  /** Only the customer with this `external_id`. */
  external_id?: string;
  /** 1 to 100, default 20. */
  limit?: number;
  cursor?: string;
}

export interface CustomerList {
  object: "list";
  data: Customer[];
  next_cursor?: string | null;
}

export type TopupStatus = "open" | "paid" | "expired";

export interface CreateTopupParams {
  /** In cents: 50 to 1000000. */
  amount: number;
  /** Only `usd` for now. */
  currency?: string;
  /** Top up this customer's balance instead of your account's. */
  customer?: string;
}

/** A payment link that adds `amount` to a balance once paid. */
export interface Topup {
  id: string;
  object: "topup";
  amount: number;
  currency: string;
  customer?: string | null;
  /** `open` until paid or `expires_at`; a payment begun in time can still turn `expired` into `paid`. */
  status: TopupStatus;
  /** Where the payer pays. */
  url: string;
  expires_at: string;
  created_at: string;
}

/**
 * One step in an intent's or a session's timeline. `type` says what `data`
 * holds: `status` (`{ status, reason? }`), `step` and `checkout` for intents,
 * `page` and `command` for sessions. New types may appear: skip the ones you
 * don't know.
 */
export interface Event {
  id: string;
  object: "event";
  type: string;
  created_at: string;
  data: Record<string, unknown>;
}

export interface EventList {
  object: "list";
  data: Event[];
  next_cursor?: string | null;
}

export interface ListEventsParams {
  /** 1 to 100, default 20. */
  limit?: number;
  cursor?: string;
}

export interface StreamEventsOptions {
  /** Start after this event (an event's `id`) instead of at the oldest. */
  lastEventId?: string;
  /** Stops the stream. */
  signal?: AbortSignal;
}

/**
 * `ready`: nothing uses it. `in_use`: an intent or a session holds it (see
 * `lease`). `forgetting`: being erased. `forgotten`: erased, and nothing saved
 * in it since.
 */
export type ProfileStatus = "ready" | "in_use" | "forgetting" | "forgotten";

/** What uses a profile now. A profile is used by one intent or session at a time. */
export interface ProfileLease {
  /** `type` is `intent` or `session`. */
  holder: { type: string; id: string };
  /** When the lease next runs out. It is renewed while the holder works. */
  expires_at: string;
}

/**
 * A payer's browser profile: the merchant logins, cookies and carts their
 * intents and sessions build up. Your account has one, and so does each
 * customer.
 */
export interface Profile {
  id: string;
  object: "profile";
  /** Its customer; `null` for your account's own profile. */
  customer?: string | null;
  status: ProfileStatus;
  lease?: ProfileLease | null;
  /** The saved profile's size; `null` before anything is saved. */
  size_bytes?: number | null;
  created_at: string;
  /** When an intent or a session last took it. */
  last_used_at?: string | null;
}

export interface CreateProfileParams {
  /** This customer's profile; without it, your account's. */
  customer?: string;
}

export interface ListProfilesParams {
  /** Only this customer's profile. */
  customer?: string;
  /** 1 to 100, default 20. */
  limit?: number;
  cursor?: string;
}

export interface ProfileList {
  object: "list";
  data: Profile[];
  next_cursor?: string | null;
}

export type SessionStatus = "queued" | "opening" | "open" | "closing" | "closed" | "failed";

export interface CreateSessionParams {
  /** The profile to open. Pass this or `customer`, not both; neither opens your account's own. */
  profile?: string;
  /** Open this customer's profile. */
  customer?: string;
  /** A public `https` URL to start on; without it, a blank page. */
  start_url?: string;
  /** 1 to 15 (the default), counted from when the session opens. */
  max_minutes?: number;
}

/** A browser opened on a profile without an intent. */
export interface Session {
  id: string;
  object: "session";
  profile: string;
  customer?: string | null;
  status: SessionStatus;
  /** Opened with a test key. */
  test: boolean;
  start_url?: string | null;
  /** The page it is on now. */
  url?: string | null;
  title?: string | null;
  created_at: string;
  opened_at?: string | null;
  /** When it closes by itself, at the latest. */
  expires_at: string;
  closed_at?: string | null;
  /** Why it ended: `released`, `idle`, `timed_out`, `data_limit`, `browser_lost` or `error`; others may appear. */
  close_reason?: string | null;
}

export interface ListSessionsParams {
  status?: SessionStatus;
  profile?: string;
  /** Only sessions on this customer's profile. */
  customer?: string;
  /** 1 to 100, default 20. */
  limit?: number;
  cursor?: string;
}

export interface SessionList {
  object: "list";
  data: Session[];
  next_cursor?: string | null;
}

/** Go to a public `https` URL; other hosts and ports are refused (`url_not_allowed`). */
export interface NavigateCommand {
  type: "navigate";
  url: string;
}

/** The page's accessibility tree, with a `ref` on each element you can act on. */
export interface SnapshotCommand {
  type: "snapshot";
}

/**
 * Act on an element by its `ref` from the latest snapshot. `type` works only
 * on text fields, with line breaks but no other control characters; `press`
 * takes one key, alone or with Shift or Control (`Enter`, `Shift+Tab`,
 * `Control+a`). After a `navigate`, and after any act except a `type`
 * without line breaks, take a new snapshot before the next act.
 */
export type ActCommand =
  | { type: "act"; ref: string; action: "click" }
  | { type: "act"; ref: string; action: "type"; text: string }
  | { type: "act"; ref: string; action: "select"; value: string }
  | { type: "act"; ref: string; action: "press"; key: string };

/** The page's readable text, or one element's with a CSS `selector`. */
export interface ReadCommand {
  type: "read";
  selector?: string;
}

/** One thing to do on a session's page. A session never pays or places an order. */
export type SessionCommand = NavigateCommand | SnapshotCommand | ActCommand | ReadCommand;

export type SessionCommandStatus = "pending" | "done" | "refused" | "failed";

/** Why a session command was `refused` or `failed`. */
export interface Failure {
  /** Such as `payment_not_allowed`, `url_not_allowed`, `not_found`, `not_actionable`, `session_closed`. */
  code: string;
  message: string;
}

export interface SessionCommandResult {
  id: string;
  object: "session_command";
  /** The command's `type`: `navigate`, `snapshot`, `act` or `read`. */
  type: string;
  status: SessionCommandStatus;
  /** The page after the command. */
  url?: string | null;
  title?: string | null;
  /** For `snapshot`, the tree. It never includes what text fields hold. */
  snapshot?: string | null;
  /** For `read`, the text. */
  text?: string | null;
  /** Why it was `refused` or `failed`. */
  error?: Failure | null;
  created_at: string;
  completed_at?: string | null;
}

export type IntentWebhookEventType =
  | "intent.running"
  | "intent.requires_payment"
  | "intent.completed"
  | "intent.failed"
  | "intent.cancelled";

/** Sent only to endpoints subscribed to `session.*` types. */
export type SessionWebhookEventType = "session.opened" | "session.closed" | "session.failed";

export type WebhookEventType = IntentWebhookEventType | SessionWebhookEventType;

export interface IntentWebhookEvent {
  id: string;
  type: IntentWebhookEventType;
  created_at: string;
  data: Intent;
}

export interface SessionWebhookEvent {
  id: string;
  type: SessionWebhookEventType;
  created_at: string;
  data: Session;
}

/** Dispatch on `type` before reading `data`. */
export type WebhookEvent = IntentWebhookEvent | SessionWebhookEvent;
