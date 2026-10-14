import type { ProfileLease } from "./types.js";

/** Error types returned by the API (`error.type` in the response body). */
export type ApiErrorType =
  | "invalid_request"
  | "authentication"
  | "payment"
  | "not_found"
  | "conflict"
  | "rate_limit"
  | "api_error";

/** Any non-2xx response from the API. */
export class OpenIntentsError extends Error {
  readonly status: number;
  readonly type: ApiErrorType | "network";
  /** A finer reason, such as `profile_busy`, `session_limit` or `session_not_open`. */
  readonly code?: string;
  readonly param?: string;
  /** With `profile_busy`: what uses the profile and when its lease next runs out. */
  readonly lease?: ProfileLease;
  readonly requestId?: string;

  constructor(opts: {
    message: string;
    status: number;
    type: ApiErrorType | "network";
    code?: string;
    param?: string;
    lease?: ProfileLease;
    requestId?: string;
  }) {
    super(opts.message);
    this.name = "OpenIntentsError";
    this.status = opts.status;
    this.type = opts.type;
    this.code = opts.code;
    this.param = opts.param;
    this.lease = opts.lease;
    this.requestId = opts.requestId;
  }
}

/** `wait()` gave up before the intent reached a final status or needed payment. */
export class WaitTimeoutError extends Error {
  constructor(readonly intentId: string, readonly timeoutMs: number) {
    super(`Intent ${intentId} did not settle within ${timeoutMs}ms`);
    this.name = "WaitTimeoutError";
  }
}
