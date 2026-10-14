use std::time::{Duration, Instant};

use rand::Rng;
use reqwest::{header, Method, RequestBuilder, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::error::{Error, Result};
use crate::stream::EventStream;
use crate::types::{
    Balance, CreateIntent, CreateProfile, CreateSession, CreateTopup, Customer, CustomerList, CustomerParams,
    EventList, Intent, IntentList, IntentStatus, ListCustomers, ListEvents, ListIntents, ListProfiles, ListSessions,
    Profile, ProfileLease, ProfileList, Session, SessionCommand, SessionCommandResult, SessionList, Topup,
};

pub const DEFAULT_BASE_URL: &str = "https://api.openintents.io/v1";
const VERSION: &str = env!("CARGO_PKG_VERSION");
/// A stream with no event and no keep-alive for this long (or the client
/// timeout, if longer) is dropped and resumed.
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

/// Client for the OpenIntents agentic API.
///
/// ```no_run
/// # async fn run() -> openintents::Result<()> {
/// use openintents::{Client, CreateIntent};
///
/// let oi = Client::from_env()?; // OPENINTENTS_API_KEY
/// let intent = oi.create_intent(CreateIntent::new("flat white, oat milk").max_amount(1000)).await?;
/// let intent = oi.wait(&intent.id, Default::default()).await?;
/// if let Some(payment) = &intent.payment {
///     println!("pay here: {}", payment.payment_url);
/// }
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
    timeout: Duration,
    pub(crate) max_retries: u32,
}

/// Options for [`Client::wait`].
#[derive(Debug, Clone, Copy)]
pub struct WaitOptions {
    /// Give up after this long. Default 5 minutes.
    pub timeout: Duration,
    /// Time between polls. Default 2s.
    pub interval: Duration,
    /// Return when the intent needs payment (default true), so a person can
    /// be sent `payment.payment_url`. False keeps waiting until it is final.
    pub return_on_payment: bool,
}

impl Default for WaitOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300),
            interval: Duration::from_secs(2),
            return_on_payment: true,
        }
    }
}

#[derive(Deserialize)]
struct ErrorBody {
    error: Option<ErrorDetail>,
}

#[derive(Deserialize)]
struct ErrorDetail {
    #[serde(rename = "type")]
    kind: Option<String>,
    code: Option<String>,
    message: Option<String>,
    param: Option<String>,
    lease: Option<ProfileLease>,
}

impl Client {
    /// A client with the given API key and default settings.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::builder(api_key).build()
    }

    /// Reads OPENINTENTS_API_KEY (and OPENINTENTS_BASE_URL, if set).
    pub fn from_env() -> Result<Self> {
        let key = std::env::var("OPENINTENTS_API_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty())
            .ok_or(Error::MissingApiKey)?;
        let mut builder = Self::builder(key);
        if let Ok(url) = std::env::var("OPENINTENTS_BASE_URL") {
            builder = builder.base_url(url);
        }
        Ok(builder.build())
    }

    pub fn builder(api_key: impl Into<String>) -> ClientBuilder {
        ClientBuilder {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 2,
        }
    }

    /// Start an intent. Returns immediately, usually with status `queued`.
    /// Sends a fresh Idempotency-Key; use [`Client::create_intent_idempotent`] to set your own.
    pub async fn create_intent(&self, params: CreateIntent) -> Result<Intent> {
        self.create_intent_idempotent(params, &uuid::Uuid::new_v4().to_string())
            .await
    }

    pub async fn create_intent_idempotent(&self, params: CreateIntent, idempotency_key: &str) -> Result<Intent> {
        self.send(Method::POST, "/intents", Some(idempotency_key), |r| r.json(&params))
            .await
    }

    pub async fn get_intent(&self, id: &str) -> Result<Intent> {
        self.send(Method::GET, &format!("/intents/{id}"), None, |r| r).await
    }

    pub async fn list_intents(&self, params: &ListIntents) -> Result<IntentList> {
        self.send(Method::GET, "/intents", None, |r| r.query(params)).await
    }

    /// Cancel an intent that hasn't paid yet. Cancelled intents are free.
    pub async fn cancel_intent(&self, id: &str) -> Result<Intent> {
        let key = uuid::Uuid::new_v4().to_string();
        self.send(Method::POST, &format!("/intents/{id}/cancel"), Some(&key), |r| r)
            .await
    }

    /// The agent balance intents pay from, and this month's spend.
    pub async fn balance(&self) -> Result<Balance> {
        self.send(Method::GET, "/balance", None, |r| r).await
    }

    /// A customer's balance.
    pub async fn balance_for(&self, customer: &str) -> Result<Balance> {
        self.send(Method::GET, "/balance", None, |r| r.query(&[("customer", customer)]))
            .await
    }

    /// Someone you buy for, with their own balance and browser profile.
    /// Sends a fresh Idempotency-Key; use [`Client::create_customer_idempotent`] to set your own.
    pub async fn create_customer(&self, params: CustomerParams) -> Result<Customer> {
        self.create_customer_idempotent(params, &uuid::Uuid::new_v4().to_string())
            .await
    }

    pub async fn create_customer_idempotent(&self, params: CustomerParams, idempotency_key: &str) -> Result<Customer> {
        self.send(Method::POST, "/customers", Some(idempotency_key), |r| r.json(&params))
            .await
    }

    pub async fn get_customer(&self, id: &str) -> Result<Customer> {
        self.send(Method::GET, &format!("/customers/{id}"), None, |r| r).await
    }

    /// Fields left out stay as they are (see [`CustomerParams`]).
    pub async fn update_customer(&self, id: &str, params: CustomerParams) -> Result<Customer> {
        self.send(Method::PATCH, &format!("/customers/{id}"), None, |r| r.json(&params))
            .await
    }

    /// Newest first.
    pub async fn list_customers(&self, params: &ListCustomers) -> Result<CustomerList> {
        self.send(Method::GET, "/customers", None, |r| r.query(params)).await
    }

    /// A payment link that adds `amount` to a balance once paid. Live keys only.
    /// Sends a fresh Idempotency-Key; use [`Client::create_topup_idempotent`] to set your own.
    pub async fn create_topup(&self, params: CreateTopup) -> Result<Topup> {
        self.create_topup_idempotent(params, &uuid::Uuid::new_v4().to_string())
            .await
    }

    pub async fn create_topup_idempotent(&self, params: CreateTopup, idempotency_key: &str) -> Result<Topup> {
        self.send(Method::POST, "/topups", Some(idempotency_key), |r| r.json(&params))
            .await
    }

    pub async fn get_topup(&self, id: &str) -> Result<Topup> {
        self.send(Method::GET, &format!("/topups/{id}"), None, |r| r).await
    }

    /// A page of the intent's timeline, oldest first: status changes, the
    /// agent's steps, its checkout.
    pub async fn list_intent_events(&self, id: &str, params: &ListEvents) -> Result<EventList> {
        self.send(Method::GET, &format!("/intents/{id}/events"), None, |r| r.query(params))
            .await
    }

    /// Follow the intent's timeline live: past events first, then new ones as
    /// they happen, after `last_event_id` if given. The stream resumes by
    /// itself after a dropped connection, and ends once the intent is over and
    /// its last event was sent.
    ///
    /// ```no_run
    /// # async fn run(oi: openintents::Client) -> openintents::Result<()> {
    /// let mut events = oi.stream_intent_events("int_7f3a2c", None);
    /// while let Some(event) = events.next().await {
    ///     let event = event?;
    ///     println!("{} {}", event.kind, event.data);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn stream_intent_events(&self, id: &str, last_event_id: Option<&str>) -> EventStream {
        EventStream::new(self.clone(), format!("/intents/{id}/events"), last_event_id)
    }

    /// Your account's profile and your customers' that exist so far, newest first.
    pub async fn list_profiles(&self, params: &ListProfiles) -> Result<ProfileList> {
        self.send(Method::GET, "/profiles", None, |r| r.query(params)).await
    }

    /// Make the profile of your account, or of a customer, ahead of its first
    /// intent or session. Optional: an intent makes the profile it needs.
    /// Making one that exists returns it unchanged, so no Idempotency-Key is
    /// sent.
    pub async fn create_profile(&self, params: CreateProfile) -> Result<Profile> {
        self.send(Method::POST, "/profiles", None, |r| r.json(&params)).await
    }

    pub async fn get_profile(&self, id: &str) -> Result<Profile> {
        self.send(Method::GET, &format!("/profiles/{id}"), None, |r| r).await
    }

    /// Erase everything the profile kept: logins, cookies, saved data. Returns
    /// it `forgetting`, then it turns `forgotten`. Forgetting twice is fine.
    pub async fn forget_profile(&self, id: &str) -> Result<Profile> {
        self.send(Method::DELETE, &format!("/profiles/{id}"), None, |r| r).await
    }

    /// Open a browser on a profile without an intent. A profile in use is a
    /// 409 `profile_busy` whose `lease` says what holds it. Sends a fresh
    /// Idempotency-Key; use [`Client::create_session_idempotent`] to set your own.
    pub async fn create_session(&self, params: CreateSession) -> Result<Session> {
        self.create_session_idempotent(params, &uuid::Uuid::new_v4().to_string())
            .await
    }

    pub async fn create_session_idempotent(&self, params: CreateSession, idempotency_key: &str) -> Result<Session> {
        self.send(Method::POST, "/sessions", Some(idempotency_key), |r| r.json(&params))
            .await
    }

    pub async fn get_session(&self, id: &str) -> Result<Session> {
        self.send(Method::GET, &format!("/sessions/{id}"), None, |r| r).await
    }

    /// Newest first.
    pub async fn list_sessions(&self, params: &ListSessions) -> Result<SessionList> {
        self.send(Method::GET, "/sessions", None, |r| r.query(params)).await
    }

    /// Close the browser, keeping what the session changed in the profile. A
    /// session that already ended is returned unchanged.
    pub async fn release_session(&self, id: &str) -> Result<Session> {
        let key = uuid::Uuid::new_v4().to_string();
        self.send(Method::POST, &format!("/sessions/{id}/release"), Some(&key), |r| r)
            .await
    }

    /// Run one command on the session's page. It waits up to 20 seconds; a
    /// command still running then comes back `Pending`, to follow with
    /// [`Client::get_session_command`]. A refused or failed one says why in
    /// `error`. Sends a fresh Idempotency-Key, so a retry never runs it twice;
    /// use [`Client::run_session_command_idempotent`] to set your own.
    pub async fn run_session_command(&self, id: &str, command: SessionCommand) -> Result<SessionCommandResult> {
        self.run_session_command_idempotent(id, command, &uuid::Uuid::new_v4().to_string())
            .await
    }

    pub async fn run_session_command_idempotent(
        &self,
        id: &str,
        command: SessionCommand,
        idempotency_key: &str,
    ) -> Result<SessionCommandResult> {
        self.send(
            Method::POST,
            &format!("/sessions/{id}/commands"),
            Some(idempotency_key),
            |r| r.json(&command),
        )
        .await
    }

    pub async fn get_session_command(&self, id: &str, command_id: &str) -> Result<SessionCommandResult> {
        self.send(
            Method::GET,
            &format!("/sessions/{id}/commands/{command_id}"),
            None,
            |r| r,
        )
        .await
    }

    /// A screenshot of the session's page, taken now: JPEG bytes.
    pub async fn session_screenshot(&self, id: &str) -> Result<Vec<u8>> {
        let res = self
            .send_raw(
                Method::GET,
                &format!("/sessions/{id}/screenshot"),
                None,
                "image/jpeg",
                |r| r,
            )
            .await?;
        Ok(res.bytes().await?.to_vec())
    }

    /// A page of the session's timeline, oldest first: status changes, pages
    /// and commands.
    pub async fn list_session_events(&self, id: &str, params: &ListEvents) -> Result<EventList> {
        self.send(Method::GET, &format!("/sessions/{id}/events"), None, |r| {
            r.query(params)
        })
        .await
    }

    /// Follow the session's timeline live, like [`Client::stream_intent_events`].
    pub fn stream_session_events(&self, id: &str, last_event_id: Option<&str>) -> EventStream {
        EventStream::new(self.clone(), format!("/sessions/{id}/events"), last_event_id)
    }

    /// Poll until the intent is final, or (by default) needs payment.
    pub async fn wait(&self, id: &str, opts: WaitOptions) -> Result<Intent> {
        let deadline = Instant::now() + opts.timeout;
        loop {
            let intent = self.get_intent(id).await?;
            if intent.status.is_final() || (opts.return_on_payment && intent.status == IntentStatus::RequiresPayment) {
                return Ok(intent);
            }
            if Instant::now() + opts.interval > deadline {
                return Err(Error::WaitTimeout {
                    intent_id: id.to_string(),
                    timeout: opts.timeout,
                });
            }
            tokio::time::sleep(opts.interval).await;
        }
    }

    async fn send<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        idempotency_key: Option<&str>,
        build: impl Fn(RequestBuilder) -> RequestBuilder,
    ) -> Result<T> {
        let res = self
            .send_raw(method, path, idempotency_key, "application/json", build)
            .await?;
        Ok(res.json().await?)
    }

    /// Sends a request, retrying 429, 5xx and connection errors with backoff,
    /// and returns the 2xx response.
    async fn send_raw(
        &self,
        method: Method,
        path: &str,
        idempotency_key: Option<&str>,
        accept: &str,
        build: impl Fn(RequestBuilder) -> RequestBuilder,
    ) -> Result<reqwest::Response> {
        let mut attempt = 0;
        loop {
            let req = self
                .request(method.clone(), path, accept, idempotency_key)
                .timeout(self.timeout);
            let res = match build(req).send().await {
                Ok(res) => res,
                Err(err) if attempt < self.max_retries && (err.is_connect() || err.is_timeout()) => {
                    tokio::time::sleep(backoff(attempt, None)).await;
                    attempt += 1;
                    continue;
                }
                Err(err) => return Err(err.into()),
            };
            let status = res.status();
            if status.is_success() {
                return Ok(res);
            }
            if retryable(status) && attempt < self.max_retries {
                tokio::time::sleep(backoff(attempt, retry_after(&res))).await;
                attempt += 1;
                continue;
            }
            return Err(api_error(res).await);
        }
    }

    /// A request with the key and the headers every call sends. Without a
    /// per-request timeout, only the client's connect and read timeouts apply,
    /// which is what an event stream needs.
    pub(crate) fn request(
        &self,
        method: Method,
        path: &str,
        accept: &str,
        idempotency_key: Option<&str>,
    ) -> RequestBuilder {
        let mut req = self
            .http
            .request(method, format!("{}{}", self.base_url, path))
            .bearer_auth(&self.api_key)
            .header(header::ACCEPT, accept);
        if let Some(key) = idempotency_key {
            req = req.header("Idempotency-Key", key);
        }
        req
    }
}

pub(crate) fn retryable(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

pub(crate) fn retry_after(res: &reqwest::Response) -> Option<f64> {
    res.headers()
        .get(header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<f64>().ok())
}

/// Turns a non-2xx response into [`Error::Api`].
pub(crate) async fn api_error(res: reqwest::Response) -> Error {
    let status = res.status();
    let request_id = res
        .headers()
        .get("request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let detail = res.json::<ErrorBody>().await.ok().and_then(|b| b.error);
    let (kind, code, param, message, lease) = match detail {
        Some(d) => (d.kind, d.code, d.param, d.message, d.lease),
        None => (None, None, None, None, None),
    };
    Error::Api {
        status: status.as_u16(),
        kind: kind.unwrap_or_else(|| {
            if status.is_server_error() {
                "api_error"
            } else {
                "invalid_request"
            }
            .into()
        }),
        code,
        param,
        message: message.unwrap_or_else(|| format!("HTTP {}", status.as_u16())),
        lease,
        request_id,
    }
}

pub(crate) fn backoff(attempt: u32, retry_after: Option<f64>) -> Duration {
    if let Some(secs) = retry_after.filter(|s| *s > 0.0) {
        return Duration::from_secs_f64(secs);
    }
    let base = (0.5 * 2f64.powi(attempt as i32)).min(8.0);
    Duration::from_secs_f64(base * rand::thread_rng().gen_range(0.75..1.25))
}

/// Configures a [`Client`].
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    api_key: String,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl ClientBuilder {
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    /// Per-request timeout. Default 30s.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Retries for 429, 5xx and connection errors. Default 2.
    pub fn max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }

    pub fn build(self) -> Client {
        // Each request sets its own total timeout, but an event stream lives
        // longer: the read timeout only drops one that stalls, as keep-alives
        // reset it.
        let http = reqwest::Client::builder()
            .connect_timeout(self.timeout)
            .read_timeout(self.timeout.max(STREAM_IDLE_TIMEOUT))
            .user_agent(format!("openintents-rust/{VERSION}"))
            .build()
            .expect("reqwest client with rustls builds");
        Client {
            http,
            api_key: self.api_key,
            base_url: self.base_url,
            timeout: self.timeout,
            max_retries: self.max_retries,
        }
    }
}
