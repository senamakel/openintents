//! Types for the OpenIntents v1 API. Source of truth: `spec/openapi.yaml`.
//! Unknown fields are ignored and unknown enum values map to `Unknown`, so new
//! API fields never break older SDK versions.

use std::collections::HashMap;

use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentStatus {
    Queued,
    Running,
    RequiresPayment,
    Completed,
    Failed,
    Cancelled,
    #[serde(other)]
    Unknown,
}

impl IntentStatus {
    /// `completed`, `failed` and `cancelled` are final.
    pub fn is_final(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::RequiresPayment => "requires_payment",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Unknown => "unknown",
        }
    }
}

/// Parameters for `POST /intents`. Build with [`CreateIntent::new`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateIntent {
    /// What to buy or book, in natural language.
    pub intent: String,
    /// Spending cap in cents; the intent fails rather than exceed it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Structured details: addresses, names, loyalty numbers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
    /// Always return a payment link instead of paying from the balance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_payment_link: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// A customer of your account to buy for: their balance and browser profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
}

impl CreateIntent {
    pub fn new(intent: impl Into<String>) -> Self {
        Self {
            intent: intent.into(),
            ..Default::default()
        }
    }

    pub fn max_amount(mut self, cents: i64) -> Self {
        self.max_amount = Some(cents);
        self
    }

    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }

    pub fn context(mut self, context: Value) -> Self {
        self.context = Some(context);
        self
    }

    pub fn require_payment_link(mut self, yes: bool) -> Self {
        self.require_payment_link = Some(yes);
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    pub fn customer(mut self, customer: impl Into<String>) -> Self {
        self.customer = Some(customer.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub reason: String,
    pub payment_url: String,
    #[serde(default)]
    pub merchant: Option<String>,
    pub total: i64,
    pub fee: i64,
    pub currency: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptItem {
    pub name: String,
    pub quantity: i64,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub merchant: String,
    #[serde(default)]
    pub items: Vec<ReceiptItem>,
    pub total: i64,
    pub fee: i64,
    pub currency: String,
    #[serde(default)]
    pub confirmation: Option<String>,
    #[serde(default)]
    pub receipt_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub id: String,
    pub status: IntentStatus,
    pub intent: String,
    /// The customer it buys for; `None` for your account's own intents.
    #[serde(default)]
    pub customer: Option<String>,
    #[serde(default)]
    pub max_amount: Option<i64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub context: Option<Value>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
    /// Set when `status` is `requires_payment`.
    #[serde(default)]
    pub payment: Option<PaymentRequest>,
    /// Set when `status` is `completed`.
    #[serde(default)]
    pub result: Option<Receipt>,
    /// Set when `status` is `failed`.
    #[serde(default)]
    pub error: Option<IntentError>,
    #[serde(default)]
    pub test: bool,
    pub created_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListIntents {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<IntentStatus>,
    /// Only intents bought for this customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentList {
    pub data: Vec<Intent>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// What intents can pay from now.
    pub available: i64,
    /// Reserved for intents at their payment step.
    #[serde(default)]
    pub held: Option<i64>,
    pub currency: String,
    /// The customer whose balance this is; `None` for your account's.
    #[serde(default)]
    pub customer: Option<String>,
    #[serde(default)]
    pub monthly_limit: Option<i64>,
    #[serde(default)]
    pub spent_this_month: Option<i64>,
}

/// Someone you buy for, with their own balance and browser profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    /// Your own ID for them, unique in your account.
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    pub created_at: String,
}

/// Fields for creating or updating a customer. On update, fields left out
/// stay as they are, an empty string clears `external_id`, `name` or
/// `email`, and `metadata` replaces the whole map.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CustomerParams {
    /// Unique in your account; one already in use is a 409 conflict.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl CustomerParams {
    pub fn external_id(mut self, id: impl Into<String>) -> Self {
        self.external_id = Some(id.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListCustomers {
    /// Only the customer with this `external_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerList {
    pub data: Vec<Customer>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopupStatus {
    /// Waiting to be paid until `expires_at`.
    Open,
    Paid,
    /// Not paid in time (a payment begun in time can still turn it `Paid`).
    Expired,
    #[serde(other)]
    Unknown,
}

/// Parameters for `POST /topups`. Build with [`CreateTopup::new`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateTopup {
    /// In cents: 50 to 1000000.
    pub amount: i64,
    /// Only `usd` for now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// Top up this customer's balance instead of your account's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
}

impl CreateTopup {
    pub fn new(amount: i64) -> Self {
        Self {
            amount,
            ..Default::default()
        }
    }

    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }

    pub fn customer(mut self, customer: impl Into<String>) -> Self {
        self.customer = Some(customer.into());
        self
    }
}

/// A payment link that adds `amount` to a balance once paid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topup {
    pub id: String,
    pub amount: i64,
    pub currency: String,
    #[serde(default)]
    pub customer: Option<String>,
    pub status: TopupStatus,
    /// Where the payer pays.
    pub url: String,
    pub expires_at: String,
    pub created_at: String,
}

/// One step in an intent's or a session's timeline. `kind` (`type` in the
/// API) says what `data` holds: `status` (`{status, reason?}`), `step` and
/// `checkout` for intents, `page` and `command` for sessions. New kinds may
/// appear: skip the ones you don't know.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub created_at: String,
    #[serde(default)]
    pub data: Value,
}

/// A page of a timeline, oldest first.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventList {
    pub data: Vec<Event>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// Paging for [`Client::list_intent_events`](crate::Client::list_intent_events)
/// and [`Client::list_session_events`](crate::Client::list_session_events).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListEvents {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileStatus {
    /// Nothing uses it.
    Ready,
    /// An intent or a session holds it (see `lease`).
    InUse,
    /// Being erased.
    Forgetting,
    /// Erased, and nothing saved in it since.
    Forgotten,
    #[serde(other)]
    Unknown,
}

/// What holds a profile: an intent or a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseHolder {
    /// `intent` or `session`.
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
}

/// What uses a profile now. A profile is used by one intent or session at a time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileLease {
    pub holder: LeaseHolder,
    /// When the lease next runs out. It is renewed while the holder works.
    pub expires_at: String,
}

/// A payer's browser profile: the merchant logins, cookies and carts their
/// intents and sessions build up. Your account has one, and so does each
/// customer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    /// Its customer; `None` for your account's own profile.
    #[serde(default)]
    pub customer: Option<String>,
    pub status: ProfileStatus,
    #[serde(default)]
    pub lease: Option<ProfileLease>,
    /// The saved profile's size; `None` before anything is saved.
    #[serde(default)]
    pub size_bytes: Option<u64>,
    pub created_at: String,
    /// When an intent or a session last took it.
    #[serde(default)]
    pub last_used_at: Option<String>,
}

/// Parameters for `POST /profiles`: your account's profile by default.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateProfile {
    /// This customer's profile instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
}

impl CreateProfile {
    pub fn customer(mut self, customer: impl Into<String>) -> Self {
        self.customer = Some(customer.into());
        self
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListProfiles {
    /// Only this customer's profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileList {
    pub data: Vec<Profile>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Queued,
    Opening,
    Open,
    Closing,
    Closed,
    Failed,
    #[serde(other)]
    Unknown,
}

impl SessionStatus {
    /// `closed` and `failed` are final.
    pub fn is_final(self) -> bool {
        matches!(self, Self::Closed | Self::Failed)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Opening => "opening",
            Self::Open => "open",
            Self::Closing => "closing",
            Self::Closed => "closed",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

/// Parameters for `POST /sessions`. The profile to open is named by
/// `profile` or by `customer`, not both; neither opens your account's own.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateSession {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
    /// A public `https` URL to start on; without it, a blank page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_url: Option<String>,
    /// 1 to 15 (the default), counted from when the session opens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_minutes: Option<u32>,
}

impl CreateSession {
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    pub fn customer(mut self, customer: impl Into<String>) -> Self {
        self.customer = Some(customer.into());
        self
    }

    pub fn start_url(mut self, url: impl Into<String>) -> Self {
        self.start_url = Some(url.into());
        self
    }

    pub fn max_minutes(mut self, minutes: u32) -> Self {
        self.max_minutes = Some(minutes);
        self
    }
}

/// A browser opened on a profile without an intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub profile: String,
    #[serde(default)]
    pub customer: Option<String>,
    pub status: SessionStatus,
    /// Opened with a test key.
    #[serde(default)]
    pub test: bool,
    #[serde(default)]
    pub start_url: Option<String>,
    /// The page it is on now.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub opened_at: Option<String>,
    /// When it closes by itself, at the latest.
    pub expires_at: String,
    #[serde(default)]
    pub closed_at: Option<String>,
    /// Why it ended: `released`, `idle`, `timed_out`, `data_limit`,
    /// `browser_lost` or `error`; others may appear.
    #[serde(default)]
    pub close_reason: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListSessions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SessionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// Only sessions on this customer's profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionList {
    pub data: Vec<Session>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// One thing to do on a session's page, for
/// [`Client::run_session_command`](crate::Client::run_session_command). A
/// session never pays or places an order: such commands come back `refused`.
///
/// ```
/// use openintents::SessionCommand;
///
/// let look = SessionCommand::snapshot();
/// let sign_in = SessionCommand::click("e12");
/// let email = SessionCommand::type_text("e7", "ada@example.com");
/// # let _ = (look, sign_in, email);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    /// Go to a public `https` URL; other hosts and ports are refused (`url_not_allowed`).
    Navigate { url: String },
    /// The page's accessibility tree, with a `ref` on each element you can act on.
    Snapshot,
    /// Act on an element by its `ref` (`element`) from the latest snapshot.
    /// After a navigate, and after any act but typing without line breaks,
    /// take a new snapshot before the next act.
    Act { element: String, action: ActAction },
    /// The page's readable text, or one element's with a CSS selector.
    Read { selector: Option<String> },
}

/// What a [`SessionCommand::Act`] does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActAction {
    Click,
    /// Type into a text field. Line breaks are fine, other control characters are not.
    Type(String),
    /// Pick this option.
    Select(String),
    /// One key, alone or with Shift or Control: `Enter`, `Shift+Tab`, `Control+a`.
    Press(String),
}

impl SessionCommand {
    pub fn navigate(url: impl Into<String>) -> Self {
        Self::Navigate { url: url.into() }
    }

    pub fn snapshot() -> Self {
        Self::Snapshot
    }

    /// The whole page's text.
    pub fn read() -> Self {
        Self::Read { selector: None }
    }

    /// One element's text, by CSS selector.
    pub fn read_selector(selector: impl Into<String>) -> Self {
        Self::Read {
            selector: Some(selector.into()),
        }
    }

    pub fn click(element: impl Into<String>) -> Self {
        Self::act(element, ActAction::Click)
    }

    pub fn type_text(element: impl Into<String>, text: impl Into<String>) -> Self {
        Self::act(element, ActAction::Type(text.into()))
    }

    pub fn select(element: impl Into<String>, value: impl Into<String>) -> Self {
        Self::act(element, ActAction::Select(value.into()))
    }

    pub fn press(element: impl Into<String>, key: impl Into<String>) -> Self {
        Self::act(element, ActAction::Press(key.into()))
    }

    fn act(element: impl Into<String>, action: ActAction) -> Self {
        Self::Act {
            element: element.into(),
            action,
        }
    }
}

impl Serialize for SessionCommand {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        match self {
            Self::Navigate { url } => {
                map.serialize_entry("type", "navigate")?;
                map.serialize_entry("url", url)?;
            }
            Self::Snapshot => map.serialize_entry("type", "snapshot")?,
            Self::Act { element, action } => {
                map.serialize_entry("type", "act")?;
                map.serialize_entry("ref", element)?;
                let (name, field) = match action {
                    ActAction::Click => ("click", None),
                    ActAction::Type(text) => ("type", Some(("text", text))),
                    ActAction::Select(value) => ("select", Some(("value", value))),
                    ActAction::Press(key) => ("press", Some(("key", key))),
                };
                map.serialize_entry("action", name)?;
                if let Some((key, value)) = field {
                    map.serialize_entry(key, value)?;
                }
            }
            Self::Read { selector } => {
                map.serialize_entry("type", "read")?;
                if let Some(selector) = selector {
                    map.serialize_entry("selector", selector)?;
                }
            }
        }
        map.end()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionCommandStatus {
    /// Still running: fetch it again by its `id`.
    Pending,
    Done,
    /// Not allowed, such as anything that would pay (`payment_not_allowed`).
    Refused,
    Failed,
    #[serde(other)]
    Unknown,
}

/// Why a session command was refused or failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    /// Such as `payment_not_allowed`, `url_not_allowed`, `not_found`,
    /// `not_actionable` or `session_closed`.
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCommandResult {
    pub id: String,
    /// The command's `type`: `navigate`, `snapshot`, `act` or `read`.
    #[serde(rename = "type")]
    pub kind: String,
    pub status: SessionCommandStatus,
    /// The page after the command.
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    /// For a snapshot, the tree. It never includes what text fields hold.
    #[serde(default)]
    pub snapshot: Option<String>,
    /// For a read, the text.
    #[serde(default)]
    pub text: Option<String>,
    /// Why it was refused or failed.
    #[serde(default)]
    pub error: Option<Failure>,
    pub created_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// A webhook delivery. Dispatch on `kind` (`type` in the API), or match on
/// `data`: `intent.*` events carry an intent, `session.*` events (sent only to
/// endpoints subscribed to them) a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "RawWebhookEvent")]
pub struct WebhookEvent {
    pub id: String,
    /// Such as `intent.completed` or `session.closed`.
    #[serde(rename = "type")]
    pub kind: String,
    pub created_at: String,
    pub data: WebhookData,
}

/// What a [`WebhookEvent`] is about.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum WebhookData {
    Intent(Box<Intent>),
    Session(Box<Session>),
    /// An event type this SDK does not know yet.
    Other(Value),
}

impl WebhookEvent {
    /// The intent, for `intent.*` events.
    pub fn intent(&self) -> Option<&Intent> {
        match &self.data {
            WebhookData::Intent(intent) => Some(intent),
            _ => None,
        }
    }

    /// The session, for `session.*` events.
    pub fn session(&self) -> Option<&Session> {
        match &self.data {
            WebhookData::Session(session) => Some(session),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
struct RawWebhookEvent {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    created_at: String,
    #[serde(default)]
    data: Value,
}

impl TryFrom<RawWebhookEvent> for WebhookEvent {
    type Error = serde_json::Error;

    fn try_from(raw: RawWebhookEvent) -> Result<Self, Self::Error> {
        let data = if raw.kind.starts_with("intent.") {
            WebhookData::Intent(Box::new(serde_json::from_value(raw.data)?))
        } else if raw.kind.starts_with("session.") {
            WebhookData::Session(Box::new(serde_json::from_value(raw.data)?))
        } else {
            WebhookData::Other(raw.data)
        };
        Ok(Self {
            id: raw.id,
            kind: raw.kind,
            created_at: raw.created_at,
            data,
        })
    }
}
