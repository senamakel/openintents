"""OpenIntents: the agentic API to buy anything.

from openintents import OpenIntents

oi = OpenIntents()  # reads OPENINTENTS_API_KEY
intent = oi.intents.create("flat white, oat milk, pickup 8:45", max_amount=1000)
intent = oi.intents.wait(intent["id"])
if intent["status"] == "requires_payment":
    print("Pay here:", intent["payment"]["payment_url"])
"""

from ._client import DEFAULT_BASE_URL, AsyncOpenIntents, OpenIntents
from ._version import __version__
from .errors import OpenIntentsError, WaitTimeoutError, WebhookVerificationError
from .types import (
    FINAL_STATUSES,
    ActCommand,
    Balance,
    Customer,
    CustomerList,
    Event,
    EventList,
    Failure,
    Intent,
    IntentList,
    IntentStatus,
    IntentWebhookEvent,
    NavigateCommand,
    PaymentRequest,
    Profile,
    ProfileLease,
    ProfileList,
    ProfileStatus,
    ReadCommand,
    Receipt,
    Session,
    SessionCommand,
    SessionCommandResult,
    SessionCommandStatus,
    SessionList,
    SessionStatus,
    SessionWebhookEvent,
    SnapshotCommand,
    Topup,
    TopupStatus,
    WebhookEvent,
)
from .webhooks import sign_webhook, verify_webhook

__all__ = [
    "DEFAULT_BASE_URL",
    "FINAL_STATUSES",
    "ActCommand",
    "AsyncOpenIntents",
    "Balance",
    "Customer",
    "CustomerList",
    "Event",
    "EventList",
    "Failure",
    "Intent",
    "IntentList",
    "IntentStatus",
    "IntentWebhookEvent",
    "NavigateCommand",
    "OpenIntents",
    "OpenIntentsError",
    "PaymentRequest",
    "Profile",
    "ProfileLease",
    "ProfileList",
    "ProfileStatus",
    "ReadCommand",
    "Receipt",
    "Session",
    "SessionCommand",
    "SessionCommandResult",
    "SessionCommandStatus",
    "SessionList",
    "SessionStatus",
    "SessionWebhookEvent",
    "SnapshotCommand",
    "Topup",
    "TopupStatus",
    "WaitTimeoutError",
    "WebhookEvent",
    "WebhookVerificationError",
    "__version__",
    "sign_webhook",
    "verify_webhook",
]
