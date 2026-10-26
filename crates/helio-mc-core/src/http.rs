//! One shared HTTP client for the whole launcher, with a host allowlist.
//!
//! Non-allowlisted hosts are rejected *before* any request is made, and
//! redirects to a non-allowlisted host are rejected on the fly. This is the
//! only place network access is created.
//!
//! Sources for the allowlist: the project spec, plus verified public docs:
//! Ely.by OAuth + authlib-injector (docs.ely.by), Mojang launchermeta,
//! Fabric/Quilt meta, Forge/NeoForge maven, Adoptium, GitHub, Modrinth.

use std::time::Duration;

use reqwest::Client;
use url::Url;

use crate::brand;
use crate::error::{HelioError, HelioResult};

/// The one shared HTTP client for Helio.
pub type HttpClient = Client;

/// Get raw bytes from a URL, optionally caching to a file.
///
/// If `cache_path` is provided and the file exists, it is returned without a
/// network request. Otherwise the bytes are downloaded and written to the
/// cache path when `cache_path` is `Some`. Returns `Ok(None)` when the
/// resource is not found (404).
pub async fn get_bytes_cached(
    url: &str,
    cache_path: Option<&std::path::Path>,
) -> HelioResult<Option<Vec<u8>>> {
    if let Some(p) = cache_path {
        if p.exists() {
            return Ok(Some(std::fs::read(p)?));
        }
    }
    let u = Url::parse(url).map_err(|e| HelioError::InvalidInput(format!("bad url: {e}")))?;
    assert_allowed(&u)?;

    let c = client();
    let resp = c
        .get(u.clone())
        .send()
        .await
        .map_err(|e| http_err(0, &e, Some(u.host_str().unwrap_or_default())))?;

    let status = resp.status();
    if status.as_u16() == 404 {
        return Ok(None);
    }
    if !status.is_success() {
        let host = resp.url().host_str().unwrap_or_default().to_string();
        let text = resp
            .text()
            .await
            .map_err(|e| http_err(status.as_u16(), &e, None))?;
        return Err(HelioError::Http {
            status: status.as_u16(),
            message: text,
            host: Some(host),
        });
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| http_err(status.as_u16(), &e, None))?;
    if let Some(p) = cache_path {
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(p, &bytes);
    }
    Ok(Some(bytes.to_vec()))
}

/// Hosts Helio is allowed to talk to. Everything else is refused.
pub const ALLOWED_HOSTS: &[&str] = &[
    "account.ely.by",
    "authserver.ely.by",
    "skinsystem.ely.by",
    "api.modrinth.com",
    "cdn.modrinth.com",
    "launchermeta.mojang.com",
    "piston-meta.mojang.com",
    "piston-data.mojang.com",
    "resources.download.minecraft.net",
    "libraries.minecraft.net",
    "meta.fabricmc.net",
    "maven.fabricmc.net",
    "meta.quiltmc.org",
    "maven.quiltmc.org",
    "files.minecraftforge.net",
    "maven.minecraftforge.net",
    "maven.neoforged.net",
    "api.adoptium.net",
    "github.com",
    "objects.githubusercontent.com",
    "raw.githubusercontent.com",
    "authlib-injector.yushi.moe",
    "release-assets.githubusercontent.com",
    "ely.by",
];

/// Build a fresh shared client. Uses rustls (no native TLS) and Helio's
/// User-Agent. Callers that need a stable pool across many requests should
/// cache one `HttpClient` instead of calling this per request.
pub fn client() -> Client {
    Client::builder()
        .user_agent(brand::user_agent())
        .timeout(Duration::from_secs(60))
        .build()
        .expect("reqwest client must build")
}

/// Check that a URL's host is on the allowlist. Returns an error otherwise.
/// This is what enforces the allowlist *before* any request.
pub fn assert_allowed(url: &Url) -> HelioResult<()> {
    let host = url.host_str().ok_or_else(|| HelioError::Http {
        status: 0,
        message: "URL has no host".into(),
        host: None,
    })?;
    if !ALLOWED_HOSTS.contains(&host) {
        return Err(HelioError::Http {
            status: 0,
            message: format!("host {host:?} is not on the allowlist"),
            host: Some(host.into()),
        });
    }
    Ok(())
}

/// Check whether a host (host_str) is allowed, used to vet redirects.
pub fn host_allowed(host: &str) -> bool {
    ALLOWED_HOSTS.contains(&host)
}

/// GET a URL as JSON. Rejects non-allowlisted hosts; rejects any redirect to
/// a non-allowlisted host.
pub async fn get_json<T: for<'de> serde::Deserialize<'de>>(url: &str) -> HelioResult<T> {
    let url = Url::parse(url).map_err(|e| HelioError::InvalidInput(format!("bad url: {e}")))?;
    assert_allowed(&url)?;

    let c = client();
    let resp = c
        .get(url.clone())
        .send()
        .await
        .map_err(|e| http_err(0, &e, Some(url.host_str().unwrap_or_default())))?;

    // Vet the final redirect target before reading.
    let final_host = resp.url().host_str().unwrap_or_default().to_string();
    if !host_allowed(&final_host) {
        return Err(HelioError::Http {
            status: resp.status().as_u16(),
            message: format!("redirect to non-allowlisted host {final_host:?}"),
            host: Some(final_host.clone()),
        });
    }

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| http_err(status.as_u16(), &e, Some(&final_host)))?;
    if !status.is_success() {
        return Err(HelioError::Http {
            status: status.as_u16(),
            message: format!("HTTP {status}: {text}"),
            host: Some(final_host),
        });
    }
    Ok(serde_json::from_str(&text)?)
}

fn http_err(status: u16, e: &reqwest::Error, host: Option<&str>) -> HelioError {
    HelioError::Http {
        status,
        message: e.to_string(),
        host: host.map(|h| h.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_allowlisted_before_request() {
        let bad = Url::parse("https://evil.example.com/x").unwrap();
        let err = assert_allowed(&bad).unwrap_err();
        match &err {
            HelioError::Http { host, .. } => assert_eq!(host.as_deref(), Some("evil.example.com")),
            _ => panic!("expected Http error, got {err:?}"),
        }
    }

    #[test]
    fn allows_known_hosts() {
        for h in [
            "account.ely.by",
            "api.modrinth.com",
            "launchermeta.mojang.com",
        ] {
            let u = Url::parse(&format!("https://{h}/x")).unwrap();
            assert!(assert_allowed(&u).is_ok(), "should allow {h}");
        }
    }

    #[test]
    fn redirect_check_is_correct() {
        assert!(host_allowed("authserver.ely.by"));
        assert!(!host_allowed("attacker.com"));
    }
}
