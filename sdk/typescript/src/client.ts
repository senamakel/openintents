import { OpenIntentsError, WaitTimeoutError, type ApiErrorType } from "./errors.js";
import { SseParser } from "./sse.js";
import {
  FINAL_STATUSES,
  type Balance,
  type BalanceParams,
  type CreateIntentParams,
  type CreateProfileParams,
  type CreateSessionParams,
  type CreateTopupParams,
  type Customer,
  type CustomerList,
  type CustomerParams,
  type Event,
  type EventList,
  type Intent,
  type IntentList,
  type ListCustomersParams,
  type ListEventsParams,
  type ListIntentsParams,
  type ListProfilesParams,
  type ListSessionsParams,
  type Profile,
  type ProfileLease,
  type ProfileList,
  type Session,
  type SessionCommand,
  type SessionCommandResult,
  type SessionList,
  type StreamEventsOptions,
  type Topup,
} from "./types.js";
import { VERSION } from "./version.js";

export const DEFAULT_BASE_URL = "https://api.openintents.io/v1";

export interface ClientOptions {
  /** Defaults to the OPENINTENTS_API_KEY environment variable. */
  apiKey?: string;
  /** Defaults to OPENINTENTS_BASE_URL, then https://api.openintents.io/v1. */
  baseUrl?: string;
  /** Per-request timeout. Default 30s. */
  timeoutMs?: number;
  /** Retries for 429, 5xx and network errors. Default 2. */
  maxRetries?: number;
  /** Custom fetch (tests, proxies, older runtimes). */
  fetch?: typeof fetch;
}

export interface RequestOptions {
  /**
   * Sent as Idempotency-Key. Every `create` (but a profile's, which is
   * idempotent by itself), `cancel`, `release` and `runCommand` generates one
   * if omitted.
   */
  idempotencyKey?: string;
  signal?: AbortSignal;
}

type Method = "GET" | "POST" | "PATCH" | "DELETE";
type Query = Record<string, string | number | undefined>;

/** A stream with no event and no keep-alive for this long (or the client timeout, if longer) is resumed. */
const STREAM_IDLE_MS = 60_000;

/** How one connection to an event stream ended, when it did not throw. */
interface Hop {
  /** The timeline is over and everything was sent (`204`). */
  over?: boolean;
  /** Set when the connection failed in a way worth retrying. */
  error?: OpenIntentsError;
  retryAfterMs?: number;
}

export interface WaitOptions {
  /** Give up after this long. Default 5 minutes. */
  timeoutMs?: number;
  /** Time between polls. Default 2s. */
  intervalMs?: number;
  /**
   * Resolve when the intent needs payment (default true), so the caller can
   * hand `intent.payment.payment_url` to a person. Set false to keep waiting
   * until it is paid and final.
   */
  returnOnPayment?: boolean;
  signal?: AbortSignal;
}

function env(name: string): string | undefined {
  // `process` is absent in browsers and some edge runtimes.
  return typeof process !== "undefined" ? process.env?.[name] : undefined;
}

const sleep = (ms: number, signal?: AbortSignal) =>
  new Promise<void>((resolve, reject) => {
    if (signal?.aborted) return reject(signal.reason);
    const t = setTimeout(resolve, ms);
    signal?.addEventListener("abort", () => {
      clearTimeout(t);
      reject(signal.reason);
    });
  });

/**
 * Client for the OpenIntents agentic API.
 *
 * ```ts
 * const oi = new OpenIntents(); // reads OPENINTENTS_API_KEY
 * const intent = await oi.intents.create({ intent: "flat white, oat milk", max_amount: 1000 });
 * const settled = await oi.intents.wait(intent.id);
 * ```
 */
export class OpenIntents {
  readonly intents: Intents;
  readonly balance: BalanceResource;
  readonly customers: Customers;
  readonly topups: Topups;
  readonly profiles: Profiles;
  readonly sessions: Sessions;

  private readonly apiKey: string;
  private readonly baseUrl: string;
  private readonly timeoutMs: number;
  private readonly maxRetries: number;
  private readonly fetchImpl: typeof fetch;

  constructor(options: ClientOptions = {}) {
    const apiKey = options.apiKey ?? env("OPENINTENTS_API_KEY");
    if (!apiKey) {
      throw new Error("Missing API key: pass { apiKey } or set OPENINTENTS_API_KEY.");
    }
    this.apiKey = apiKey;
    this.baseUrl = (options.baseUrl ?? env("OPENINTENTS_BASE_URL") ?? DEFAULT_BASE_URL).replace(/\/+$/, "");
    this.timeoutMs = options.timeoutMs ?? 30_000;
    this.maxRetries = options.maxRetries ?? 2;
    this.fetchImpl = options.fetch ?? globalThis.fetch.bind(globalThis);
    this.intents = new Intents(this);
    this.balance = new BalanceResource(this);
    this.customers = new Customers(this);
    this.topups = new Topups(this);
    this.profiles = new Profiles(this);
    this.sessions = new Sessions(this);
  }

  /** @internal Makes a request and parses the JSON answer. */
  async request<T>(
    method: Method,
    path: string,
    opts: { body?: unknown; query?: Query } & RequestOptions = {},
  ): Promise<T> {
    return (await (await this.send(method, path, opts)).json()) as T;
  }

  /** @internal Sends a request, retrying 429/5xx/network errors with backoff, and returns the 2xx response. */
  async send(
    method: Method,
    path: string,
    opts: { body?: unknown; query?: Query; accept?: string } & RequestOptions = {},
  ): Promise<Response> {
    const url = this.url(path, opts.query);
    const headers = this.headers(opts.accept ?? "application/json");
    if (opts.body !== undefined) headers["Content-Type"] = "application/json";
    if (opts.idempotencyKey) headers["Idempotency-Key"] = opts.idempotencyKey;

    for (let attempt = 0; ; attempt++) {
      const timeout = AbortSignal.timeout(this.timeoutMs);
      const signal = opts.signal ? AbortSignal.any([opts.signal, timeout]) : timeout;
      let res: Response;
      try {
        res = await this.fetchImpl(url, {
          method,
          headers,
          body: opts.body === undefined ? undefined : JSON.stringify(opts.body),
          signal,
        });
      } catch (err) {
        if (opts.signal?.aborted) throw err;
        if (attempt < this.maxRetries) {
          await sleep(backoff(attempt));
          continue;
        }
        throw networkError(err);
      }

      if (res.ok) return res;

      if (retryable(res.status) && attempt < this.maxRetries) {
        await sleep(retryAfterMs(res) ?? backoff(attempt));
        continue;
      }
      throw await toError(res);
    }
  }

  /**
   * @internal Follows a timeline over Server-Sent Events: past events first,
   * then new ones as they happen. A stream that closes or drops is resumed
   * after the last event received (`Last-Event-ID`) until the API answers
   * `204`: the timeline is over and every event was sent.
   */
  async *stream(path: string, opts: StreamEventsOptions = {}): AsyncGenerator<Event, void, undefined> {
    const state = { lastEventId: opts.lastEventId, failures: 0 };
    for (;;) {
      const hop = yield* this.follow(path, state, opts.signal);
      if (hop.over) return;
      // The API closes a stream now and then (resuming picks it up): reconnect at once.
      if (!hop.error) continue;
      if (state.failures >= this.maxRetries) throw hop.error;
      await sleep(hop.retryAfterMs ?? backoff(state.failures), opts.signal);
      state.failures++;
    }
  }

  /** One connection to an event stream: yields its events, then says how it ended. */
  private async *follow(
    path: string,
    state: { lastEventId?: string; failures: number },
    signal?: AbortSignal,
  ): AsyncGenerator<Event, Hop, undefined> {
    if (signal?.aborted) throw signal.reason;
    const ctrl = new AbortController();
    let reader: ReadableStreamDefaultReader<Uint8Array> | undefined;
    // Cancels the body as well as the request: a custom fetch may not tie the two.
    const close = (reason: unknown) => {
      ctrl.abort(reason);
      reader?.cancel(reason).catch(() => undefined);
    };
    const stop = () => close(signal?.reason);
    signal?.addEventListener("abort", stop, { once: true });
    // Drops a connection that stalls: no answer, or no event or keep-alive, in time.
    let stalled: Error | undefined;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const arm = (ms: number) => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        stalled = new Error(`the event stream sent nothing for ${ms}ms`);
        close(stalled);
      }, ms);
    };
    try {
      arm(this.timeoutMs);
      const headers = this.headers("text/event-stream");
      if (state.lastEventId !== undefined) headers["Last-Event-ID"] = state.lastEventId;
      let res: Response;
      try {
        res = await this.fetchImpl(this.url(path), { method: "GET", headers, signal: ctrl.signal });
      } catch (err) {
        if (signal?.aborted) throw signal.reason;
        return { error: networkError(stalled ?? err) };
      }
      if (res.status === 204) return { over: true };
      if (!res.ok) {
        const error = await toError(res);
        if (!retryable(res.status)) throw error;
        return { error, retryAfterMs: retryAfterMs(res) };
      }
      if (!res.body) return { error: networkError(new Error("the event stream has no body")) };

      reader = res.body.getReader();
      const decoder = new TextDecoder();
      const parser = new SseParser();
      let sentAnything = false;
      for (;;) {
        arm(Math.max(this.timeoutMs, STREAM_IDLE_MS));
        let chunk: ReadableStreamReadResult<Uint8Array>;
        try {
          chunk = await reader.read();
        } catch (err) {
          if (signal?.aborted) throw signal.reason;
          return { error: networkError(stalled ?? err) };
        }
        if (signal?.aborted) throw signal.reason;
        if (stalled) return { error: networkError(stalled) };
        if (chunk.value?.length) {
          // Anything arriving, a keep-alive too, shows the stream works.
          sentAnything = true;
          state.failures = 0;
        }
        const messages = chunk.done
          ? [...parser.push(decoder.decode()), ...parser.end()]
          : parser.push(decoder.decode(chunk.value, { stream: true }));
        for (const message of messages) {
          const event = JSON.parse(message.data) as Event;
          state.lastEventId = message.id ?? event.id;
          yield event;
        }
        if (chunk.done) {
          // A connection that closes before sending a byte counts as a failure, so it cannot spin.
          return sentAnything ? {} : { error: networkError(new Error("the event stream closed empty")) };
        }
      }
    } finally {
      clearTimeout(timer);
      signal?.removeEventListener("abort", stop);
      // Closes the connection when the caller stops early.
      close(new Error("the event stream was closed"));
    }
  }

  private url(path: string, query: Query = {}): URL {
    const url = new URL(this.baseUrl + path);
    for (const [k, v] of Object.entries(query)) {
      if (v !== undefined) url.searchParams.set(k, String(v));
    }
    return url;
  }

  private headers(accept: string): Record<string, string> {
    return {
      Authorization: `Bearer ${this.apiKey}`,
      Accept: accept,
      "User-Agent": `openintents-typescript/${VERSION}`,
    };
  }
}

function backoff(attempt: number) {
  return Math.min(8000, 500 * 2 ** attempt) * (0.75 + Math.random() * 0.5);
}

function retryable(status: number) {
  return status === 429 || status >= 500;
}

function retryAfterMs(res: Response): number | undefined {
  const after = Number(res.headers.get("retry-after"));
  return Number.isFinite(after) && after > 0 ? after * 1000 : undefined;
}

function networkError(err: unknown): OpenIntentsError {
  return new OpenIntentsError({ message: `Network error: ${(err as Error).message}`, status: 0, type: "network" });
}

async function toError(res: Response): Promise<OpenIntentsError> {
  let body: {
    error?: { type?: ApiErrorType; code?: string; message?: string; param?: string; lease?: ProfileLease };
  } = {};
  try {
    body = (await res.json()) as typeof body;
  } catch {
    // Non-JSON error body; fall through to a generic message.
  }
  return new OpenIntentsError({
    message: body.error?.message ?? `HTTP ${res.status}`,
    status: res.status,
    type: body.error?.type ?? (res.status >= 500 ? "api_error" : "invalid_request"),
    code: body.error?.code,
    param: body.error?.param,
    lease: body.error?.lease ?? undefined,
    requestId: res.headers.get("request-id") ?? undefined,
  });
}

class Intents {
  constructor(private readonly client: OpenIntents) {}

  /** Start an intent. Returns immediately, usually with `status: "queued"`. */
  create(params: CreateIntentParams, opts: RequestOptions = {}): Promise<Intent> {
    return this.client.request<Intent>("POST", "/intents", {
      body: params,
      ...opts,
      idempotencyKey: opts.idempotencyKey ?? crypto.randomUUID(),
    });
  }

  get(id: string, opts: RequestOptions = {}): Promise<Intent> {
    return this.client.request<Intent>("GET", `/intents/${encodeURIComponent(id)}`, opts);
  }

  list(params: ListIntentsParams = {}, opts: RequestOptions = {}): Promise<IntentList> {
    return this.client.request<IntentList>("GET", "/intents", { query: { ...params }, ...opts });
  }

  /** Iterate every intent matching `params`, following `next_cursor`. */
  async *listAll(params: Omit<ListIntentsParams, "cursor"> = {}): AsyncGenerator<Intent> {
    let cursor: string | undefined;
    do {
      const page = await this.list({ ...params, cursor });
      yield* page.data;
      cursor = page.next_cursor ?? undefined;
    } while (cursor);
  }

  /** Cancel an intent that hasn't paid yet. Cancelled intents are free. */
  cancel(id: string, opts: RequestOptions = {}): Promise<Intent> {
    return this.client.request<Intent>("POST", `/intents/${encodeURIComponent(id)}/cancel`, {
      ...opts,
      idempotencyKey: opts.idempotencyKey ?? crypto.randomUUID(),
    });
  }

  /** A page of the intent's timeline, oldest first: status changes, the agent's steps, its checkout. */
  listEvents(id: string, params: ListEventsParams = {}, opts: RequestOptions = {}): Promise<EventList> {
    return this.client.request<EventList>("GET", `/intents/${encodeURIComponent(id)}/events`, {
      query: { ...params },
      ...opts,
    });
  }

  /**
   * Follow the intent's timeline live: past events first, then new ones as
   * they happen. Resumes by itself after a dropped connection, and ends once
   * the intent is over and its last event was sent.
   *
   * ```ts
   * for await (const event of oi.intents.streamEvents(id)) console.log(event.type, event.data);
   * ```
   */
  streamEvents(id: string, opts: StreamEventsOptions = {}): AsyncGenerator<Event, void, undefined> {
    return this.client.stream(`/intents/${encodeURIComponent(id)}/events`, opts);
  }

  /**
   * Poll until the intent is final (`completed`, `failed`, `cancelled`) or, by
   * default, needs payment (`requires_payment`, with `payment.payment_url`).
   */
  async wait(id: string, opts: WaitOptions = {}): Promise<Intent> {
    const timeoutMs = opts.timeoutMs ?? 300_000;
    const intervalMs = opts.intervalMs ?? 2_000;
    const returnOnPayment = opts.returnOnPayment ?? true;
    const deadline = Date.now() + timeoutMs;
    for (;;) {
      const intent = await this.get(id, { signal: opts.signal });
      if (FINAL_STATUSES.includes(intent.status)) return intent;
      if (returnOnPayment && intent.status === "requires_payment") return intent;
      if (Date.now() + intervalMs > deadline) throw new WaitTimeoutError(id, timeoutMs);
      await sleep(intervalMs, opts.signal);
    }
  }
}

class BalanceResource {
  constructor(private readonly client: OpenIntents) {}

  /** The agent balance intents pay from, and this month's spend; a customer's with `{ customer }`. */
  get(opts: BalanceParams & RequestOptions = {}): Promise<Balance> {
    const { customer, ...rest } = opts;
    return this.client.request<Balance>("GET", "/balance", { query: { customer }, ...rest });
  }
}

class Customers {
  constructor(private readonly client: OpenIntents) {}

  /** Someone you buy for, with their own balance and browser profile. */
  create(params: CustomerParams = {}, opts: RequestOptions = {}): Promise<Customer> {
    return this.client.request<Customer>("POST", "/customers", {
      body: params,
      ...opts,
      idempotencyKey: opts.idempotencyKey ?? crypto.randomUUID(),
    });
  }

  get(id: string, opts: RequestOptions = {}): Promise<Customer> {
    return this.client.request<Customer>("GET", `/customers/${encodeURIComponent(id)}`, opts);
  }

  /**
   * Fields left out stay as they are; an empty string clears `external_id`,
   * `name` or `email`; `metadata` replaces the whole map.
   */
  update(id: string, params: CustomerParams, opts: RequestOptions = {}): Promise<Customer> {
    return this.client.request<Customer>("PATCH", `/customers/${encodeURIComponent(id)}`, {
      body: params,
      ...opts,
    });
  }

  /** Newest first. */
  list(params: ListCustomersParams = {}, opts: RequestOptions = {}): Promise<CustomerList> {
    return this.client.request<CustomerList>("GET", "/customers", { query: { ...params }, ...opts });
  }

  /** Iterate every customer matching `params`, following `next_cursor`. */
  async *listAll(params: Omit<ListCustomersParams, "cursor"> = {}): AsyncGenerator<Customer> {
    let cursor: string | undefined;
    do {
      const page = await this.list({ ...params, cursor });
      yield* page.data;
      cursor = page.next_cursor ?? undefined;
    } while (cursor);
  }
}

class Topups {
  constructor(private readonly client: OpenIntents) {}

  /** A payment link that adds `amount` to a balance once paid. Live keys only. */
  create(params: CreateTopupParams, opts: RequestOptions = {}): Promise<Topup> {
    return this.client.request<Topup>("POST", "/topups", {
      body: params,
      ...opts,
      idempotencyKey: opts.idempotencyKey ?? crypto.randomUUID(),
    });
  }

  get(id: string, opts: RequestOptions = {}): Promise<Topup> {
    return this.client.request<Topup>("GET", `/topups/${encodeURIComponent(id)}`, opts);
  }
}

class Profiles {
  constructor(private readonly client: OpenIntents) {}

  /** Your account's profile and your customers' that exist so far, newest first. */
  list(params: ListProfilesParams = {}, opts: RequestOptions = {}): Promise<ProfileList> {
    return this.client.request<ProfileList>("GET", "/profiles", { query: { ...params }, ...opts });
  }

  /** Iterate every profile matching `params`, following `next_cursor`. */
  async *listAll(params: Omit<ListProfilesParams, "cursor"> = {}): AsyncGenerator<Profile> {
    let cursor: string | undefined;
    do {
      const page = await this.list({ ...params, cursor });
      yield* page.data;
      cursor = page.next_cursor ?? undefined;
    } while (cursor);
  }

  /**
   * Make the profile of your account, or of `customer`, ahead of its first
   * intent or session. Optional: an intent makes the profile it needs. Making
   * one that exists returns it unchanged, so no Idempotency-Key is needed.
   */
  create(params: CreateProfileParams = {}, opts: RequestOptions = {}): Promise<Profile> {
    return this.client.request<Profile>("POST", "/profiles", { body: params, ...opts });
  }

  get(id: string, opts: RequestOptions = {}): Promise<Profile> {
    return this.client.request<Profile>("GET", `/profiles/${encodeURIComponent(id)}`, opts);
  }

  /**
   * Erase everything the profile kept: logins, cookies, saved data. Returns it
   * `forgetting`, then it turns `forgotten`. Forgetting twice is fine.
   */
  forget(id: string, opts: RequestOptions = {}): Promise<Profile> {
    return this.client.request<Profile>("DELETE", `/profiles/${encodeURIComponent(id)}`, opts);
  }
}

class Sessions {
  constructor(private readonly client: OpenIntents) {}

  /**
   * Open a browser on a profile without an intent: your account's with `{}`,
   * or `{ profile }` or `{ customer }`. A profile in use throws a 409
   * `profile_busy` whose `lease` says what holds it.
   */
  create(params: CreateSessionParams = {}, opts: RequestOptions = {}): Promise<Session> {
    return this.client.request<Session>("POST", "/sessions", {
      body: params,
      ...opts,
      idempotencyKey: opts.idempotencyKey ?? crypto.randomUUID(),
    });
  }

  get(id: string, opts: RequestOptions = {}): Promise<Session> {
    return this.client.request<Session>("GET", `/sessions/${encodeURIComponent(id)}`, opts);
  }

  /** Newest first. */
  list(params: ListSessionsParams = {}, opts: RequestOptions = {}): Promise<SessionList> {
    return this.client.request<SessionList>("GET", "/sessions", { query: { ...params }, ...opts });
  }

  /** Iterate every session matching `params`, following `next_cursor`. */
  async *listAll(params: Omit<ListSessionsParams, "cursor"> = {}): AsyncGenerator<Session> {
    let cursor: string | undefined;
    do {
      const page = await this.list({ ...params, cursor });
      yield* page.data;
      cursor = page.next_cursor ?? undefined;
    } while (cursor);
  }

  /**
   * Close the browser, keeping what the session changed in the profile. A
   * session that already ended is returned unchanged.
   */
  release(id: string, opts: RequestOptions = {}): Promise<Session> {
    return this.client.request<Session>("POST", `/sessions/${encodeURIComponent(id)}/release`, {
      ...opts,
      idempotencyKey: opts.idempotencyKey ?? crypto.randomUUID(),
    });
  }

  /**
   * Run one command on the session's page. It waits up to 20 seconds; a
   * command still running then comes back `pending`, to follow with
   * `getCommand`. A refused or failed one says why in `error`.
   *
   * ```ts
   * const tree = await oi.sessions.runCommand(id, { type: "snapshot" });
   * await oi.sessions.runCommand(id, { type: "act", ref: "e12", action: "click" });
   * ```
   */
  runCommand(id: string, command: SessionCommand, opts: RequestOptions = {}): Promise<SessionCommandResult> {
    return this.client.request<SessionCommandResult>("POST", `/sessions/${encodeURIComponent(id)}/commands`, {
      body: command,
      ...opts,
      idempotencyKey: opts.idempotencyKey ?? crypto.randomUUID(),
    });
  }

  getCommand(id: string, commandId: string, opts: RequestOptions = {}): Promise<SessionCommandResult> {
    return this.client.request<SessionCommandResult>(
      "GET",
      `/sessions/${encodeURIComponent(id)}/commands/${encodeURIComponent(commandId)}`,
      opts,
    );
  }

  /** A screenshot of the session's page, taken now: JPEG bytes. */
  async screenshot(id: string, opts: RequestOptions = {}): Promise<Uint8Array> {
    const res = await this.client.send("GET", `/sessions/${encodeURIComponent(id)}/screenshot`, {
      ...opts,
      accept: "image/jpeg",
    });
    return new Uint8Array(await res.arrayBuffer());
  }

  /** A page of the session's timeline, oldest first: status changes, pages and commands. */
  listEvents(id: string, params: ListEventsParams = {}, opts: RequestOptions = {}): Promise<EventList> {
    return this.client.request<EventList>("GET", `/sessions/${encodeURIComponent(id)}/events`, {
      query: { ...params },
      ...opts,
    });
  }

  /**
   * Follow the session's timeline live: past events first, then new ones as
   * they happen. Resumes by itself after a dropped connection, and ends once
   * the session is over and its last event was sent.
   */
  streamEvents(id: string, opts: StreamEventsOptions = {}): AsyncGenerator<Event, void, undefined> {
    return this.client.stream(`/sessions/${encodeURIComponent(id)}/events`, opts);
  }
}
