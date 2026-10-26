//! Ely.by OAuth2 login.
//!
//! Endpoints and scopes verified against the current Ely.by developer docs
//! (https://docs.ely.by/en/oauth.html, 2026-10-26):
//!
//! - Authorize:  `https://account.ely.by/oauth2/v1`
//! - Token:      `https://account.ely.by/api/oauth2/v1/token`
//! - User info:  `https://account.ely.by/api/account/v1/info`
//! - Scopes:     `account_info`, `account_email`, `offline_access`
//!               (returns a `refresh_token`), `minecraft_server_session`
//!               (lets the access token be used as a Minecraft session id).
//!
//! Ely.by's token endpoint requires `client_id` + `client_secret`, so a
//! client secret is mandatory for native apps. The secret is read from
//! `--client-secret`, `HELIO_ELYBY_CLIENT_SECRET`, or
//! `config.elyby_client_secret` — never hard-coded.
//!
//! The flow for a CLI:
//! 1. Open the authorize URL in the system browser (and always print it).
//! 2. Run a one-shot HTTP listener on `127.0.0.1:<port>` that captures the
//!     redirect. With `--no-browser`, the user pastes the final URL or code
//!     back instead.
//! 3. Exchange the code for tokens, fetch the profile, store tokens in the
//!     keychain, and save the account metadata.
//! 4. `ensure_fresh_token` refreshes when the access token is within 5
//!     minutes of expiry.
//!
//! PKCE is NOT supported by Ely.by's OAuth server (the docs show no
//! `code_challenge` parameter), so the flow is authorization-code with
//! client-secret.

use std::sync::Arc;
use std::time::Duration;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{HelioError, HelioResult};
use crate::http;

/// The base URLs for Ely.by OAuth (verified, do not invent).
pub const AUTHORIZE_URL: &str = "https://account.ely.by/oauth2/v1";
pub const TOKEN_URL: &str = "https://account.ely.by/api/oauth2/v1/token";
pub const USER_INFO_URL: &str = "https://account.ely.by/api/account/v1/info";

/// The scopes to request. `offline_access` is required so we get a
/// refresh_token; `minecraft_server_session` lets the access token be used
/// as the game's session id.
pub const SCOPES: &str = "account_info offline_access minecraft_server_session";

/// The redirect URIs we register with Ely.by. Ely.by requires an exact match
/// against a registered redirect, so we use a fixed, documented port with a
/// short fallback list. Users register ONE of these when they create their
/// "Helio" app at `account.ely.by/dev/applications`.
///
/// Register at least the first: `http://127.0.0.1:8765/callback`.
pub const CALLBACK_PORTS: &[u16] = &[8765, 8766, 8767];

/// How long the listener waits for the user to complete authorization.
pub const AUTH_TIMEOUT: Duration = Duration::from_secs(240);

/// A user profile returned by the Ely.by user-info endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub uuid: String,
    pub username: String,
}

/// A token response from the Ely.by token endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_in: i64,
}

/// Everything the front end needs to drive the login flow.
pub struct AuthFlow {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub state: String,
    pub port: u16,
}

/// Resolve the client id + secret from the config. Fails with exact setup
/// instructions when missing.
pub fn credentials_from_config(config: &Config) -> HelioResult<(String, String)> {
    let client_id = config
        .elyby_client_id
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| setup_error("elyby_client_id"))?;
    let client_secret = config
        .elyby_client_secret
        .clone()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| setup_error("elyby_client_secret"))?;
    Ok((client_id, client_secret))
}

/// Build a fresh auth-flow context (random state, pick a callback port).
pub fn begin(client_id: &str, client_secret: &str, port: Option<u16>) -> AuthFlow {
    let port = port.unwrap_or(CALLBACK_PORTS[0]);
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let state: String = {
        let mut b = [0u8; 24];
        use rand::RngCore;
        rand::thread_rng().fill_bytes(&mut b);
        URL_SAFE_NO_PAD.encode(b)
    };
    AuthFlow {
        client_id: client_id.to_string(),
        client_secret: client_secret.to_string(),
        redirect_uri,
        state,
        port,
    }
}

/// Build the authorize URL for the browser.
pub fn authorize_url(flow: &AuthFlow) -> String {
    let mut q = url::form_urlencoded::Serializer::new(String::new());
    q.append_pair("client_id", &flow.client_id);
    q.append_pair("redirect_uri", &flow.redirect_uri);
    q.append_pair("response_type", "code");
    q.append_pair("scope", SCOPES);
    q.append_pair("state", &flow.state);
    format!("{AUTHORIZE_URL}?{}", q.finish())
}

/// The one-shot redirect listener. Runs until it captures a matching
/// `state`, the user errors, or the timeout elapses. Returns the auth code.
///
/// Uses a tiny hand-rolled HTTP/1.1 reader over a `TcpListener` so the
/// response is fully under our control (we emit a success page, and we can
/// cancel on Ctrl+C without the port being held by a lingering request).
pub async fn listen_for_code(
    flow: &AuthFlow,
    cancel: Arc<tokio::sync::Notify>,
) -> HelioResult<String> {
    use tokio::io::AsyncWriteExt;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], flow.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| HelioError::Http {
            status: 0,
            message: format!("could not bind 127.0.0.1:{}: {e}", flow.port),
            host: Some("127.0.0.1".into()),
        })?;

    let deadline = tokio::time::sleep(AUTH_TIMEOUT);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => {
                return Err(HelioError::Cancelled);
            }
            _ = cancel.notified() => {
                // Ctrl+C: drop the listener to free the port.
                return Err(HelioError::Cancelled);
            }
            accept = listener.accept() => {
                let (mut stream, _) = accept.map_err(|e| HelioError::Http {
                    status: 0,
                    message: format!("accept failed: {e}"),
                    host: None,
                })?;

                // Read the request head + body until \r\n\r\n.
                let req = read_http_request(&mut stream).await;
                let Some(req) = req else {
                    continue; // connection closed mid-read; keep waiting.
                };

                let url = extract_request_url(&req)?;
                let query = url.query().unwrap_or("");

                if let Some(err) = query_param(query, "error") {
                    let desc = query_param(query, "error_description").unwrap_or_default();
                    return Err(HelioError::Auth(format!(
                        "Ely.by returned error {err:?}: {desc}"
                    )));
                }

                let state = match query_param(query, "state") {
                    Some(s) => s,
                    None => {
                        respond(&mut stream, "no state").await;
                        continue;
                    }
                };
                if state != flow.state {
                    // Tampered or replayed state: reject and keep waiting.
                    respond(&mut stream, "state mismatch").await;
                    continue;
                }
                let code = match query_param(query, "code") {
                    Some(c) => c,
                    None => {
                        respond(&mut stream, "no code").await;
                        continue;
                    }
                };

                // Success page.
                let html = r#"<!doctype html><html><head><title>Helio</title></head>
<body style="font-family:monospace;background:#161c28;color:#f0f0eb">
<div style="padding:3rem">
  <pre style="color:#ffb000">HELIO</pre>
  <h2>Signed in.</h2>
  <p>You can return to the terminal and close this tab.</p>
</div></body></html>"#;
                let bytes = html.as_bytes();
                let _ = stream
                    .write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                            bytes.len()
                        )
                        .as_bytes(),
                    )
                    .await;
                let _ = stream.write_all(bytes).await;
                return Ok(code.to_string());
            }
        }
    }
}

/// Read a minimal HTTP request (head + body) until the headers end.
async fn read_http_request(stream: &mut tokio::net::TcpStream) -> Option<Vec<u8>> {
    use tokio::io::AsyncReadExt;
    let mut buf = [0u8; 8192];
    let mut n = 0;
    loop {
        match stream.read(&mut buf[n..]).await {
            Ok(0) => break,
            Ok(k) => {
                n += k;
                let head_end = std::str::from_utf8(&buf[..n])
                    .ok()
                    .and_then(|s| s.find("\r\n\r\n"));
                if head_end.is_some() {
                    break;
                }
                if n == buf.len() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Some(buf[..n].to_vec())
}

/// Parse the request line + headers into a URL we can query.
fn extract_request_url(req: &Vec<u8>) -> HelioResult<url::Url> {
    let text = String::from_utf8_lossy(req).to_string();
    let request_line = text.lines().next().unwrap_or("");
    // GET /callback?code=...&state=... HTTP/1.1
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(HelioError::Http {
            status: 0,
            message: "malformed HTTP request".into(),
            host: Some("127.0.0.1".into()),
        });
    }
    let path = parts[1];
    // Reconstruct an absolute URL so .query() works.
    let full = format!("http://127.0.0.1{path}");
    url::Url::parse(&full).map_err(|e| HelioError::Http {
        status: 0,
        message: format!("bad redirect url {full:?}: {e}"),
        host: Some("127.0.0.1".into()),
    })
}

fn query_param(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        if let Some((k, v)) = url::form_urlencoded::parse(pair.as_bytes()).next() {
            if k == key {
                return Some(v.to_string());
            }
        }
    }
    None
}

async fn respond(stream: &mut tokio::net::TcpStream, reason: &str) {
    use tokio::io::AsyncWriteExt;
    let body = format!(
        r#"<!doctype html><body style="font-family:monospace"><p>Helio: {reason}.
Please return to the terminal and try again.</p></body>"#
    );
    let _ = stream
        .write_all(
            format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n",
                body.len()
            )
            .as_bytes(),
        )
        .await;
    let _ = stream.write_all(body.as_bytes()).await;
}

/// Exchange an auth code for tokens.
pub async fn exchange_code(
    client_id: &str,
    client_secret: &str,
    redirect_uri: &str,
    code: &str,
) -> HelioResult<TokenResponse> {
    let form = serde_urlencoded::to_string(&[
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", redirect_uri),
        ("code", code),
    ])?;
    let resp: TokenResponse = http_post_json(TOKEN_URL, &form, None).await?;
    if resp.access_token.is_empty() {
        return Err(HelioError::Auth(
            "empty access_token in token response".into(),
        ));
    }
    Ok(resp)
}

/// Refresh an access token using a refresh_token.
pub async fn refresh(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
    scope: &str,
) -> HelioResult<TokenResponse> {
    let form = serde_urlencoded::to_string(&[
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("scope", scope),
        ("refresh_token", refresh_token),
    ])?;
    http_post_json(TOKEN_URL, &form, None).await
}

/// Fetch the user profile (uuid + username) for an access token.
pub async fn fetch_profile(access_token: &str) -> HelioResult<UserProfile> {
    let raw: serde_json::Value = http_get_json_with_auth(USER_INFO_URL, access_token).await?;
    let uuid = raw
        .get("uuid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HelioError::Auth("user-info response missing `uuid`".into()))?;
    let username = raw
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HelioError::Auth("user-info response missing `username`".into()))?;
    Ok(UserProfile {
        uuid: uuid.to_string(),
        username: username.to_string(),
    })
}

/// Refresh a stored Ely.by account's tokens when they are about to expire.
///
/// `expires_in` from the token endpoint is a lifetime in seconds; we record
/// `expires_at = now + expires_in` at login time. Refresh when
/// `expires_at - now < 5*60`.
pub fn token_is_stale(secrets: &crate::accounts::keychain::ElybySecrets) -> bool {
    if secrets.expires_at <= 0 {
        return false; // no recorded expiry; treat as valid
    }
    let now = chrono::Utc::now().timestamp();
    secrets.expires_at - now < 300
}

/// POST to a URL with a form body, returning JSON. `auth` is an optional
/// Bearer token.
async fn http_post_json<T: serde::de::DeserializeOwned>(
    url: &str,
    form: &str,
    auth: Option<&str>,
) -> HelioResult<T> {
    let client = http::client();
    let mut req = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form.to_string());
    if let Some(t) = auth {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let resp = req.send().await.map_err(|e| http_err(0, &e, url))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| http_err(status.as_u16(), &e, url))?;
    if !status.is_success() {
        let desc = parse_error_field(&text);
        return Err(HelioError::Auth(format!(
            "HTTP {}: {desc}",
            status.as_u16()
        )));
    }
    Ok(serde_json::from_str(&text)?)
}

/// GET JSON with a Bearer token.
async fn http_get_json_with_auth<T: serde::de::DeserializeOwned>(
    url: &str,
    access_token: &str,
) -> HelioResult<T> {
    let client = http::client();
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await
        .map_err(|e| http_err(0, &e, url))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| http_err(status.as_u16(), &e, url))?;
    if !status.is_success() {
        let desc = parse_error_field(&text);
        return Err(HelioError::Auth(format!(
            "HTTP {}: {desc}",
            status.as_u16()
        )));
    }
    Ok(serde_json::from_str(&text)?)
}

fn http_err(status: u16, e: &reqwest::Error, url: &str) -> HelioError {
    let host = url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(String::from));
    HelioError::Http {
        status,
        message: e.to_string(),
        host,
    }
}

/// Extract the `error` / `errorMessage` field from an Ely.by error body.
fn parse_error_field(text: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(m) = v.get("errorMessage").and_then(|m| m.as_str()) {
            return m.to_string();
        }
        if let Some(e) = v.get("error").and_then(|m| m.as_str()) {
            return e.to_string();
        }
    }
    text.to_string()
}

/// Human instructions when credentials are missing.
fn setup_error(missing: &str) -> HelioError {
    HelioError::Auth(format!(
        "missing {missing}. To set it up:\n  1. Register an app named \"Helio\" at \
https://account.ely.by/dev/applications\n  2. Add redirect URI \
http://127.0.0.1:{}/callback (and optionally the fallback ports)\n  3. Copy the \
client id and secret into Helio's config: helio config set elyby_client_id <ID> \
and helio config set elyby_client_secret <SECRET>\n  (or pass --client-id / \
--client-secret)",
        CALLBACK_PORTS[0]
    ))
}

/// A convenience: run the whole browser flow and return the profile.
///
/// `no_browser`: when true, the caller already printed the URL and the user
/// will paste the redirect URL or code back via `complete_from_paste`.
pub async fn run_browser_flow(
    flow: &AuthFlow,
    cancel: Arc<tokio::sync::Notify>,
) -> HelioResult<(UserProfile, TokenResponse)> {
    let code = listen_for_code(flow, cancel).await?;
    let tokens = exchange_code(
        &flow.client_id,
        &flow.client_secret,
        &flow.redirect_uri,
        &code,
    )
    .await?;
    let profile = fetch_profile(&tokens.access_token).await?;
    Ok((profile, tokens))
}

/// Complete a login from a pasted redirect URL or bare code (the
/// `--no-browser` / SSH path). Validates the state when a full URL is given.
pub async fn complete_from_paste(
    flow: &AuthFlow,
    pasted: &str,
) -> HelioResult<(UserProfile, TokenResponse)> {
    let code = parse_paste(pasted, &flow.state)?;
    let tokens = exchange_code(
        &flow.client_id,
        &flow.client_secret,
        &flow.redirect_uri,
        &code,
    )
    .await?;
    let profile = fetch_profile(&tokens.access_token).await?;
    Ok((profile, tokens))
}

/// Parse a pasted value that may be the full redirect URL
/// (`http://127.0.0.1:8765/callback?code=...&state=...`) or just the code.
/// When it is a URL, validate the state matches.
fn parse_paste(pasted: &str, expected_state: &str) -> HelioResult<String> {
    let trimmed = pasted.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        let url = url::Url::parse(trimmed)
            .map_err(|e| HelioError::InvalidInput(format!("pasted value is not a URL: {e}")))?;
        let query = url.query().unwrap_or("");
        if let Some(s) = query_param(query, "state") {
            if s != expected_state {
                return Err(HelioError::Auth(
                    "state mismatch: the pasted URL is from a different login session".into(),
                ));
            }
        }
        let code = query_param(query, "code").ok_or_else(|| {
            HelioError::InvalidInput("pasted URL has no `code` query parameter".into())
        })?;
        return Ok(code.to_string());
    }
    // Assume it is the bare code.
    if trimmed.is_empty() {
        return Err(HelioError::InvalidInput("empty paste".into()));
    }
    Ok(trimmed.to_string())
}

/// The token expiry, in unix seconds, recorded at login time.
pub fn token_expires_at(tokens: &TokenResponse) -> i64 {
    let now = chrono::Utc::now().timestamp();
    let ttl = tokens.expires_in.max(0);
    now + ttl
}

/// Open a URL in the default system browser (best-effort; the URL is also
/// printed for copy-paste).
pub fn open_in_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_paste_with_valid_state() {
        let flow = begin("id", "secret", None);
        let url = format!(
            "http://127.0.0.1:{}/callback?code=abc123&state={}",
            flow.port, flow.state
        );
        assert_eq!(parse_paste(&url, &flow.state).unwrap(), "abc123");
    }

    #[test]
    fn parse_paste_rejects_tampered_state() {
        let flow = begin("id", "secret", None);
        let url = format!(
            "http://127.0.0.1:{}/callback?code=abc123&state=TAMPERED",
            flow.port
        );
        assert!(parse_paste(&url, &flow.state).is_err());
    }

    #[test]
    fn parse_paste_with_bare_code() {
        let flow = begin("id", "secret", None);
        assert_eq!(
            parse_paste("just-a-code", &flow.state).unwrap(),
            "just-a-code"
        );
    }

    #[test]
    fn authorize_url_has_all_params() {
        let flow = begin("cid", "csecret", Some(8765));
        let url = authorize_url(&flow);
        assert!(url.starts_with(AUTHORIZE_URL));
        assert!(url.contains("client_id=cid"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("scope="));
        assert!(url.contains(&format!("state={}", flow.state)));
        assert!(url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A8765%2Fcallback"));
    }

    #[test]
    fn token_stale_detection() {
        use crate::accounts::keychain::ElybySecrets;
        let now = chrono::Utc::now().timestamp();
        let fresh = ElybySecrets {
            access_token: "x".into(),
            refresh_token: None,
            expires_at: now + 3600,
            scope: "account_info".into(),
        };
        assert!(!token_is_stale(&fresh));
        let stale = ElybySecrets {
            access_token: "x".into(),
            refresh_token: None,
            expires_at: now + 60,
            scope: "account_info".into(),
        };
        assert!(token_is_stale(&stale));
        let no_expiry = ElybySecrets {
            access_token: "x".into(),
            refresh_token: None,
            expires_at: 0,
            scope: "account_info".into(),
        };
        assert!(!token_is_stale(&no_expiry));
    }
}
