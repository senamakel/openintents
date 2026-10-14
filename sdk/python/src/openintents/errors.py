from __future__ import annotations

from .types import ProfileLease


class OpenIntentsError(Exception):
    """Any non-2xx response from the API, or a network failure (status 0).

    `code` gives a finer reason, such as "profile_busy" or "session_not_open";
    with "profile_busy", `lease` says what uses the profile and when its lease
    next runs out.
    """

    def __init__(
        self,
        message: str,
        *,
        status: int,
        type: str,
        code: str | None = None,
        param: str | None = None,
        lease: ProfileLease | None = None,
        request_id: str | None = None,
    ) -> None:
        super().__init__(message)
        self.message = message
        self.status = status
        self.type = type
        self.code = code
        self.param = param
        self.lease = lease
        self.request_id = request_id

    def __repr__(self) -> str:
        return (
            f"OpenIntentsError(status={self.status}, type={self.type!r}, "
            f"code={self.code!r}, message={self.message!r})"
        )


class WaitTimeoutError(TimeoutError):
    """`wait()` gave up before the intent reached a final status or needed payment."""

    def __init__(self, intent_id: str, timeout: float) -> None:
        super().__init__(f"Intent {intent_id} did not settle within {timeout}s")
        self.intent_id = intent_id
        self.timeout = timeout


class WebhookVerificationError(ValueError):
    """The webhook signature is missing, malformed, wrong or too old."""
