"""Types for the OpenIntents v1 API. Source of truth: spec/openapi.yaml.

Responses are returned as plain dicts typed with TypedDicts, so new fields
from the API never break older SDK versions.
"""

from __future__ import annotations

from typing import Any, Literal, TypedDict

IntentStatus = Literal["queued", "running", "requires_payment", "completed", "failed", "cancelled"]

#: Statuses an intent never leaves.
FINAL_STATUSES: frozenset[str] = frozenset({"completed", "failed", "cancelled"})


class PaymentRequest(TypedDict, total=False):
    reason: Literal["insufficient_balance", "over_limit", "requested"]
    payment_url: str
    merchant: str | None
    total: int
    fee: int
    currency: str
    expires_at: str


class ReceiptItem(TypedDict):
    name: str
    quantity: int
    amount: int


class Receipt(TypedDict, total=False):
    merchant: str
    items: list[ReceiptItem]
    total: int
    fee: int
    currency: str
    confirmation: str | None
    receipt_url: str | None


class IntentError(TypedDict):
    code: str
    message: str


class Intent(TypedDict, total=False):
    id: str
    object: Literal["intent"]
    status: IntentStatus
    intent: str
    customer: str | None
    max_amount: int | None
    currency: str | None
    context: dict[str, Any] | None
    metadata: dict[str, str] | None
    payment: PaymentRequest | None
    result: Receipt | None
    error: IntentError | None
    test: bool
    created_at: str
    completed_at: str | None


class IntentList(TypedDict, total=False):
    object: Literal["list"]
    data: list[Intent]
    next_cursor: str | None


class Balance(TypedDict, total=False):
    object: Literal["balance"]
    available: int
    held: int
    currency: str
    customer: str | None
    monthly_limit: int | None
    spent_this_month: int


class Customer(TypedDict, total=False):
    """Someone you buy for, with their own balance and browser profile."""

    id: str
    object: Literal["customer"]
    external_id: str | None
    name: str | None
    email: str | None
    metadata: dict[str, str]
    created_at: str


class CustomerList(TypedDict, total=False):
    object: Literal["list"]
    data: list[Customer]
    next_cursor: str | None


TopupStatus = Literal["open", "paid", "expired"]


class Topup(TypedDict, total=False):
    """A payment link that adds `amount` to a balance once paid."""

    id: str
    object: Literal["topup"]
    amount: int
    currency: str
    customer: str | None
    status: TopupStatus
    url: str
    expires_at: str
    created_at: str


class Event(TypedDict, total=False):
    """One step in an intent's or a session's timeline.

    `type` says what `data` holds: "status" ({status, reason?}), "step" and
    "checkout" for intents, "page" and "command" for sessions. New types may
    appear: skip the ones you don't know.
    """

    id: str
    object: Literal["event"]
    type: str
    created_at: str
    data: dict[str, Any]


class EventList(TypedDict, total=False):
    object: Literal["list"]
    data: list[Event]
    next_cursor: str | None


#: "ready": nothing uses it. "in_use": an intent or a session holds it (see
#: lease). "forgetting": being erased. "forgotten": erased, nothing saved since.
ProfileStatus = Literal["ready", "in_use", "forgetting", "forgotten"]


class LeaseHolder(TypedDict):
    type: str
    id: str


class ProfileLease(TypedDict):
    """What uses a profile now. A profile is used by one intent or session at a time."""

    holder: LeaseHolder
    expires_at: str


class Profile(TypedDict, total=False):
    """A payer's browser profile: the merchant logins, cookies and carts they build up."""

    id: str
    object: Literal["profile"]
    customer: str | None
    status: ProfileStatus
    lease: ProfileLease | None
    size_bytes: int | None
    created_at: str
    last_used_at: str | None


class ProfileList(TypedDict, total=False):
    object: Literal["list"]
    data: list[Profile]
    next_cursor: str | None


SessionStatus = Literal["queued", "opening", "open", "closing", "closed", "failed"]


class Session(TypedDict, total=False):
    """A browser opened on a profile without an intent."""

    id: str
    object: Literal["session"]
    profile: str
    customer: str | None
    status: SessionStatus
    test: bool
    start_url: str | None
    url: str | None
    title: str | None
    created_at: str
    opened_at: str | None
    expires_at: str
    closed_at: str | None
    #: "released", "idle", "timed_out", "data_limit", "browser_lost" or "error"; others may appear.
    close_reason: str | None


class SessionList(TypedDict, total=False):
    object: Literal["list"]
    data: list[Session]
    next_cursor: str | None


class NavigateCommand(TypedDict):
    """Go to a public https URL; other hosts and ports are refused (url_not_allowed)."""

    type: Literal["navigate"]
    url: str


class SnapshotCommand(TypedDict):
    """The page's accessibility tree, with a ref on each element you can act on."""

    type: Literal["snapshot"]


class _ActCommand(TypedDict):
    type: Literal["act"]
    ref: str
    action: Literal["click", "type", "select", "press"]


class ActCommand(_ActCommand, total=False):
    """Act on an element by its ref from the latest snapshot.

    "type" takes `text` (text fields only; line breaks but no other control
    characters), "select" takes `value`, "press" takes `key` (one key, alone
    or with Shift or Control: "Enter", "Shift+Tab", "Control+a"). After a
    navigate, and after any act but a "type" without line breaks, take a new
    snapshot before the next act.
    """

    text: str
    value: str
    key: str


class _ReadCommand(TypedDict):
    type: Literal["read"]


class ReadCommand(_ReadCommand, total=False):
    """The page's readable text, or one element's with a CSS selector."""

    selector: str


#: One thing to do on a session's page. A session never pays or places an order.
SessionCommand = NavigateCommand | SnapshotCommand | ActCommand | ReadCommand

SessionCommandStatus = Literal["pending", "done", "refused", "failed"]


class Failure(TypedDict):
    """Why a session command was refused or failed, with a code such as payment_not_allowed."""

    code: str
    message: str


class SessionCommandResult(TypedDict, total=False):
    id: str
    object: Literal["session_command"]
    type: str
    status: SessionCommandStatus
    url: str | None
    title: str | None
    #: For a snapshot, the tree. It never includes what text fields hold.
    snapshot: str | None
    #: For a read, the text.
    text: str | None
    error: Failure | None
    created_at: str
    completed_at: str | None


IntentWebhookEventType = Literal[
    "intent.running",
    "intent.requires_payment",
    "intent.completed",
    "intent.failed",
    "intent.cancelled",
]

#: Sent only to endpoints subscribed to session.* types.
SessionWebhookEventType = Literal["session.opened", "session.closed", "session.failed"]


class IntentWebhookEvent(TypedDict):
    id: str
    type: IntentWebhookEventType
    created_at: str
    data: Intent


class SessionWebhookEvent(TypedDict):
    id: str
    type: SessionWebhookEventType
    created_at: str
    data: Session


#: Dispatch on "type" before reading "data".
WebhookEvent = IntentWebhookEvent | SessionWebhookEvent
