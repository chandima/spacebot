//! OpenAI ChatGPT Plus OAuth: device code flow, browser PKCE flow, token exchange, refresh, and storage.

use anyhow::{Context as _, Result};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use std::path::{Path, PathBuf};

const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const OAUTH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const DEVICE_USERCODE_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/usercode";
const DEVICE_TOKEN_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/token";
const DEVICE_REDIRECT_URI: &str = "https://auth.openai.com/deviceauth/callback";
const DEFAULT_DEVICE_VERIFICATION_URL: &str = "https://auth.openai.com/codex/device";

const BROWSER_OAUTH_PORT: u16 = 1455;
const BROWSER_REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const BROWSER_OAUTH_AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
const BROWSER_OAUTH_SCOPES: &str = "openid profile email offline_access";

/// Stored OpenAI OAuth credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
    pub access_token: String,
    pub refresh_token: String,
    /// Expiry as Unix timestamp in milliseconds.
    pub expires_at: i64,
    pub account_id: Option<String>,
}

impl OAuthCredentials {
    /// Check if the access token is expired or about to expire (within 5 minutes).
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        let buffer = 5 * 60 * 1000;
        now >= self.expires_at - buffer
    }

    /// Refresh the access token and return updated credentials.
    pub async fn refresh(&self) -> Result<Self> {
        let client = reqwest::Client::new();
        let response = client
            .post(OAUTH_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", self.refresh_token.as_str()),
                ("client_id", CLIENT_ID),
            ])
            .send()
            .await
            .context("failed to send OpenAI OAuth refresh request")?;

        let status = response.status();
        let body = response
            .text()
            .await
            .context("failed to read OpenAI OAuth refresh response")?;

        if !status.is_success() {
            anyhow::bail!("OpenAI OAuth refresh failed ({}): {}", status, body);
        }

        let token_response: TokenResponse =
            serde_json::from_str(&body).context("failed to parse OpenAI OAuth refresh response")?;

        let account_id = extract_account_id(&token_response).or_else(|| self.account_id.clone());
        let refresh_token = token_response
            .refresh_token
            .unwrap_or_else(|| self.refresh_token.clone());

        Ok(Self {
            access_token: token_response.access_token,
            refresh_token,
            expires_at: chrono::Utc::now().timestamp_millis()
                + token_response.expires_in.unwrap_or(3600) * 1000,
            account_id,
        })
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenClaims {
    chatgpt_account_id: Option<String>,
    organizations: Option<Vec<TokenOrganization>>,
    #[serde(rename = "https://api.openai.com/auth")]
    openai_auth: Option<TokenOpenAiAuthClaims>,
}

#[derive(Debug, Deserialize)]
struct TokenOrganization {
    id: String,
}

#[derive(Debug, Deserialize)]
struct TokenOpenAiAuthClaims {
    chatgpt_account_id: Option<String>,
}

fn parse_jwt_claims(token: &str) -> Option<TokenClaims> {
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let payload = parts.next()?;
    let _signature = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice::<TokenClaims>(&decoded).ok()
}

fn extract_account_id(token_response: &TokenResponse) -> Option<String> {
    let from_claims = |claims: TokenClaims| {
        claims
            .chatgpt_account_id
            .or_else(|| claims.openai_auth.and_then(|auth| auth.chatgpt_account_id))
            .or_else(|| {
                claims
                    .organizations
                    .and_then(|organizations| organizations.into_iter().next())
                    .map(|organization| organization.id)
            })
    };

    token_response
        .id_token
        .as_deref()
        .and_then(parse_jwt_claims)
        .and_then(from_claims)
        .or_else(|| parse_jwt_claims(&token_response.access_token).and_then(from_claims))
}

fn deserialize_optional_u64<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<u64>, D::Error> {
    use serde::de::Error;

    let value: Option<serde_json::Value> = Option::deserialize(d)?;
    match value {
        None => Ok(None),
        Some(serde_json::Value::Number(number)) => number
            .as_u64()
            .map(Some)
            .ok_or_else(|| Error::custom("expected positive integer")),
        Some(serde_json::Value::String(value)) => value
            .parse()
            .map(Some)
            .map_err(|error| Error::custom(format!("invalid integer: {error}"))),
        Some(other) => Err(Error::custom(format!(
            "expected string or number, got {other}"
        ))),
    }
}

/// Response from the OpenAI device-code usercode endpoint.
#[derive(Debug, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_auth_id: String,
    pub user_code: String,
    /// Recommended polling interval in seconds (API may return this as a string).
    #[serde(default, deserialize_with = "deserialize_optional_u64")]
    pub interval: Option<u64>,
    /// Time in seconds before the device code expires (API may return this as a string).
    #[serde(default, deserialize_with = "deserialize_optional_u64")]
    pub expires_in: Option<u64>,
    #[serde(default, alias = "verification_uri", alias = "verification_url")]
    pub verification_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceTokenSuccessResponse {
    authorization_code: String,
    code_verifier: String,
}

#[derive(Debug, Deserialize)]
struct DeviceTokenErrorResponse {
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeviceTokenGrant {
    pub authorization_code: String,
    pub code_verifier: String,
}

#[derive(Debug, Clone)]
pub enum DeviceTokenPollResult {
    Pending,
    SlowDown,
    Approved(DeviceTokenGrant),
}

/// Step 1: Request a device code and user code from OpenAI.
pub async fn request_device_code() -> Result<DeviceCodeResponse> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({ "client_id": CLIENT_ID });

    let response = client
        .post(DEVICE_USERCODE_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("failed to send OpenAI device-code usercode request")?;

    let status = response.status();
    let text = response
        .text()
        .await
        .context("failed to read OpenAI device-code usercode response")?;

    if status == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!(
            "Device code login is not enabled. Please enable it in your ChatGPT security settings at https://chatgpt.com/security-settings and try again."
        );
    }

    if !status.is_success() {
        anyhow::bail!(
            "OpenAI device-code usercode request failed ({}): {}",
            status,
            text
        );
    }

    serde_json::from_str::<DeviceCodeResponse>(&text)
        .context("failed to parse OpenAI device-code usercode response")
}

/// Step 2: Poll the device token endpoint once.
pub async fn poll_device_token(
    device_auth_id: &str,
    user_code: &str,
) -> Result<DeviceTokenPollResult> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "device_auth_id": device_auth_id,
        "user_code": user_code,
    });

    let response = client
        .post(DEVICE_TOKEN_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .context("failed to send OpenAI device-code token poll request")?;

    let status = response.status();
    if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::NOT_FOUND {
        return Ok(DeviceTokenPollResult::Pending);
    }

    let body = response
        .text()
        .await
        .context("failed to read OpenAI device-code token poll response")?;

    if status.is_success() {
        let device_token: DeviceTokenSuccessResponse = serde_json::from_str(&body)
            .context("failed to parse OpenAI device-code token poll response")?;

        return Ok(DeviceTokenPollResult::Approved(DeviceTokenGrant {
            authorization_code: device_token.authorization_code,
            code_verifier: device_token.code_verifier,
        }));
    }

    if (status == reqwest::StatusCode::BAD_REQUEST
        || status == reqwest::StatusCode::TOO_MANY_REQUESTS)
        && let Ok(error_response) = serde_json::from_str::<DeviceTokenErrorResponse>(&body)
    {
        if matches!(
            error_response.error.as_deref(),
            Some("authorization_pending")
        ) {
            return Ok(DeviceTokenPollResult::Pending);
        }
        if matches!(error_response.error.as_deref(), Some("slow_down")) {
            return Ok(DeviceTokenPollResult::SlowDown);
        }
        if let Some(description) = error_response.error_description.as_deref() {
            anyhow::bail!(
                "OpenAI device-code token poll failed ({}): {}",
                status,
                description
            );
        }
    }

    anyhow::bail!(
        "OpenAI device-code token poll failed ({}): {}",
        status,
        body
    );
}

/// Determine which verification URL to show the user.
pub fn device_verification_url(response: &DeviceCodeResponse) -> String {
    response
        .verification_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| DEFAULT_DEVICE_VERIFICATION_URL.to_string())
}

/// Step 3: Exchange the device authorization code for OAuth tokens.
pub async fn exchange_device_code(
    authorization_code: &str,
    code_verifier: &str,
) -> Result<OAuthCredentials> {
    let client = reqwest::Client::new();
    let response = client
        .post(OAUTH_TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", authorization_code),
            ("redirect_uri", DEVICE_REDIRECT_URI),
            ("client_id", CLIENT_ID),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await
        .context("failed to send OpenAI device-code token exchange request")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("failed to read OpenAI device-code token exchange response")?;

    if !status.is_success() {
        anyhow::bail!(
            "OpenAI device-code token exchange failed ({}): {}",
            status,
            body
        );
    }

    let token_response: TokenResponse = serde_json::from_str(&body)
        .context("failed to parse OpenAI device-code token exchange response")?;
    let account_id = extract_account_id(&token_response);
    let refresh_token = token_response
        .refresh_token
        .context("OpenAI device-code token response did not include refresh_token")?;

    Ok(OAuthCredentials {
        access_token: token_response.access_token,
        refresh_token,
        expires_at: chrono::Utc::now().timestamp_millis()
            + token_response.expires_in.unwrap_or(3600) * 1000,
        account_id,
    })
}

// ---------------------------------------------------------------------------
// Browser PKCE flow (works with SSO / enterprise logins)
// ---------------------------------------------------------------------------

/// PKCE verifier and challenge pair.
struct Pkce {
    verifier: String,
    challenge: String,
}

/// Generate a PKCE code verifier (43 chars) and S256 challenge.
fn generate_pkce() -> Pkce {
    // 32 random bytes → 43-char base64url string
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let verifier = URL_SAFE_NO_PAD.encode(bytes);

    let hash = Sha256::digest(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hash);

    Pkce {
        verifier,
        challenge,
    }
}

/// Generate a random CSRF state token.
fn generate_state() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Build the browser authorization URL with PKCE parameters.
fn browser_authorize_url(pkce: &Pkce, state: &str) -> String {
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&code_challenge={}&code_challenge_method=S256&state={}&id_token_add_organizations=true&codex_cli_simplified_flow=true&originator=spacebot",
        BROWSER_OAUTH_AUTHORIZE_URL,
        CLIENT_ID,
        urlencoding::encode(BROWSER_REDIRECT_URI),
        urlencoding::encode(BROWSER_OAUTH_SCOPES),
        pkce.challenge,
        state,
    )
}

/// Exchange a browser authorization code for OAuth tokens (PKCE flow).
async fn exchange_browser_code(code: &str, code_verifier: &str) -> Result<OAuthCredentials> {
    let client = reqwest::Client::new();
    let response = client
        .post(OAUTH_TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", BROWSER_REDIRECT_URI),
            ("client_id", CLIENT_ID),
            ("code_verifier", code_verifier),
        ])
        .send()
        .await
        .context("failed to send OpenAI browser PKCE token exchange request")?;

    let status = response.status();
    let body = response
        .text()
        .await
        .context("failed to read OpenAI browser PKCE token exchange response")?;

    if !status.is_success() {
        anyhow::bail!(
            "OpenAI browser PKCE token exchange failed ({}): {}",
            status,
            body
        );
    }

    let token_response: TokenResponse = serde_json::from_str(&body)
        .context("failed to parse OpenAI browser PKCE token exchange response")?;
    let account_id = extract_account_id(&token_response);
    let refresh_token = token_response
        .refresh_token
        .context("OpenAI browser PKCE response did not include refresh_token")?;

    Ok(OAuthCredentials {
        access_token: token_response.access_token,
        refresh_token,
        expires_at: chrono::Utc::now().timestamp_millis()
            + token_response.expires_in.unwrap_or(3600) * 1000,
        account_id,
    })
}

/// Run the interactive browser PKCE login flow.
///
/// 1. Generates PKCE verifier/challenge and CSRF state
/// 2. Starts a temporary HTTP server on localhost:1455
/// 3. Opens the browser to OpenAI's authorization page
/// 4. Waits for the callback with the authorization code
/// 5. Exchanges the code for tokens
/// 6. Saves credentials to disk
pub async fn login_browser_interactive(instance_dir: &Path) -> Result<OAuthCredentials> {
    let pkce = generate_pkce();
    let state = generate_state();
    let authorize_url = browser_authorize_url(&pkce, &state);

    // Channel to receive the authorization code from the callback handler
    let (code_tx, mut code_rx) = tokio::sync::mpsc::channel::<Result<String>>(1);

    let expected_state = state.clone();
    let callback_handler = {
        let code_tx = code_tx.clone();
        move |uri: axum::http::Uri| {
            let code_tx = code_tx.clone();
            let expected_state = expected_state.clone();
            async move {
                let query = uri.query().unwrap_or("");
                let params: std::collections::HashMap<String, String> =
                    url::form_urlencoded::parse(query.as_bytes())
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect();

                if let Some(error) = params.get("error") {
                    let description = params
                        .get("error_description")
                        .map(|d| d.as_str())
                        .unwrap_or("unknown error");
                    code_tx
                        .send(Err(anyhow::anyhow!("OAuth error: {} — {}", error, description)))
                        .await
                        .ok();
                    return axum::response::Html(
                        "<html><body><h2>Authentication failed</h2><p>You can close this tab.</p></body></html>"
                            .to_string(),
                    );
                }

                let received_state = params.get("state").map(|s| s.as_str()).unwrap_or("");
                if received_state != expected_state {
                    code_tx
                        .send(Err(anyhow::anyhow!("invalid state — potential CSRF")))
                        .await
                        .ok();
                    return axum::response::Html(
                        "<html><body><h2>Invalid state</h2><p>You can close this tab.</p></body></html>"
                            .to_string(),
                    );
                }

                if let Some(code) = params.get("code") {
                    code_tx.send(Ok(code.clone())).await.ok();
                    axum::response::Html(
                        "<html><body><h2>✓ Authenticated!</h2><p>You can close this tab and return to your terminal.</p><script>setTimeout(()=>window.close(),2000)</script></body></html>"
                            .to_string(),
                    )
                } else {
                    code_tx
                        .send(Err(anyhow::anyhow!("missing authorization code")))
                        .await
                        .ok();
                    axum::response::Html(
                        "<html><body><h2>Missing code</h2><p>You can close this tab.</p></body></html>"
                            .to_string(),
                    )
                }
            }
        }
    };

    let app = axum::Router::new().route(
        "/auth/callback",
        axum::routing::get(callback_handler),
    );

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{BROWSER_OAUTH_PORT}"))
        .await
        .with_context(|| {
            format!("failed to bind to localhost:{BROWSER_OAUTH_PORT} — is another process using it?")
        })?;

    eprintln!("\nOpening browser for OpenAI authentication...\n");
    eprintln!("If the browser doesn't open, visit:\n  {authorize_url}\n");

    if let Err(_error) = open::that(&authorize_url) {
        eprintln!("(Could not open browser automatically)");
    }

    eprintln!("Waiting for authorization (timeout: 5 minutes)...");

    // Run server with a 5-minute timeout
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .ok();
    });

    let code = tokio::select! {
        result = code_rx.recv() => {
            match result {
                Some(Ok(code)) => code,
                Some(Err(error)) => {
                    server_handle.abort();
                    return Err(error);
                }
                None => {
                    server_handle.abort();
                    anyhow::bail!("callback channel closed unexpectedly");
                }
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
            server_handle.abort();
            anyhow::bail!("authorization timed out after 5 minutes");
        }
    };

    // Shut down the callback server
    server_handle.abort();

    eprintln!("Exchanging authorization code for tokens...");

    let creds = exchange_browser_code(&code, &pkce.verifier)
        .await
        .context("failed to exchange browser authorization code")?;

    save_credentials(instance_dir, &creds)?;

    eprintln!(
        "\n✓ OpenAI ChatGPT authenticated via browser.\n  Credentials saved to {}",
        credentials_path(instance_dir).display()
    );

    Ok(creds)
}

/// Path to OpenAI OAuth credentials within the instance directory.
pub fn credentials_path(instance_dir: &Path) -> PathBuf {
    instance_dir.join("openai_chatgpt_oauth.json")
}

/// Load OpenAI OAuth credentials from disk.
pub fn load_credentials(instance_dir: &Path) -> Result<Option<OAuthCredentials>> {
    let path = credentials_path(instance_dir);
    if !path.exists() {
        return Ok(None);
    }

    let data = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let creds: OAuthCredentials =
        serde_json::from_str(&data).context("failed to parse OpenAI OAuth credentials")?;
    Ok(Some(creds))
}

/// Save OpenAI OAuth credentials to disk with restricted permissions (0600).
pub fn save_credentials(instance_dir: &Path, creds: &OAuthCredentials) -> Result<()> {
    let path = credentials_path(instance_dir);
    let data = serde_json::to_string_pretty(creds)
        .context("failed to serialize OpenAI OAuth credentials")?;

    std::fs::write(&path, &data).with_context(|| format!("failed to write {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("failed to set permissions on {}", path.display()))?;
    }

    Ok(())
}
