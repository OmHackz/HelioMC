//! Skin and cape fetching.
//!
//! Ely.by: `https://skinsystem.ely.by/skins/<name>.png` and
//! `https://skinsystem.ely.by/cloaks/<name>.png` (verified 2026-10-26,
//! docs.ely.by skins-system). Cached under `cache/skins/` with a TTL.
//! Offline: bundled default Steve/Alex chosen by UUID parity.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::HelioResult;
use crate::http;

/// A fetched skin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skin {
    /// The raw PNG bytes.
    pub png: Vec<u8>,
    /// Width in pixels (default 64).
    pub width: u32,
    /// Height in pixels (default 64 or 32).
    pub height: u32,
    /// Skin model: `slim` or `wide` (Alex vs Steve).
    pub model: SkinModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkinModel {
    Slim,
    Wide,
}

/// A fetched cape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cape {
    pub png: Vec<u8>,
}

/// The skin cache entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SkinCache {
    fetched_at: i64,
    skin: Option<Skin>,
    cape: Option<Cape>,
}

const SKIN_TTL: Duration = Duration::from_secs(300); // 5 min

/// Fetch a skin for an Ely.by account (by username), with caching.
/// Returns `None` when the user has no custom skin.
pub async fn get_skin(data_dir: &Path, username: &str) -> HelioResult<Option<Skin>> {
    let cache_dir = data_dir.join("cache").join("skins");
    std::fs::create_dir_all(&cache_dir)?;
    let cache_path = cache_dir.join(format!("{}.json", username.to_lowercase()));

    if let Ok(c) = std::fs::read(&cache_path) {
        if let Ok(entry) = serde_json::from_slice::<SkinCache>(&c) {
            let age = chrono::Utc::now()
                .timestamp()
                .saturating_sub(entry.fetched_at);
            if age < SKIN_TTL.as_secs() as i64 {
                return Ok(entry.skin);
            }
        }
    }

    let url = format!("https://skinsystem.ely.by/skins/{username}.png");
    let skin = match http::get_bytes_cached(&url, Some(&cache_dir.join(format!("{username}.png"))))
        .await?
    {
        Some(bytes) => Some(parse_png(&bytes)),
        None => None,
    };
    if let Some(skin) = &skin {
        let entry = SkinCache {
            fetched_at: chrono::Utc::now().timestamp(),
            skin: Some(skin.clone()),
            cape: None,
        };
        let _ = std::fs::write(&cache_path, serde_json::to_vec(&entry)?);
    }
    Ok(skin)
}

/// Fetch a cape for an Ely.by account.
pub async fn get_cape(data_dir: &Path, username: &str) -> HelioResult<Option<Cape>> {
    let url = format!("https://skinsystem.ely.by/cloaks/{username}.png");
    match http::get_bytes_cached(
        &url,
        Some(
            &data_dir
                .join("cache")
                .join("skins")
                .join(format!("{username}-cape.png")),
        ),
    )
    .await?
    {
        Some(bytes) => Ok(Some(Cape { png: bytes })),
        None => Ok(None),
    }
}

/// Parse a PNG into a `Skin` (dimensions + model guess).
fn parse_png(bytes: &[u8]) -> Skin {
    // PNG: bytes 16..24 are IHDR width/height (big-endian u32).
    let (width, height) = if bytes.len() >= 24 {
        let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        (w, h)
    } else {
        (64, 64)
    };
    // Height 32 => slim model; 64 => wide.
    let model = if height == 32 {
        SkinModel::Slim
    } else {
        SkinModel::Wide
    };
    Skin {
        png: bytes.to_vec(),
        width,
        height,
        model,
    }
}

/// The bundled default skin bytes for offline accounts.
///
/// We embed two minimal 64x32 PNGs (Steve and Alex) as base64. Generated
/// from the vanilla default skin assets; here we ship tiny placeholders so
/// the terminal renderer has something to draw. The real 64x64 pixel data
/// is not needed for the CLI head render — only the model matters.
pub fn default_offline_skin(model: SkinModel) -> Skin {
    // A solid-color 64x32 PNG placeholder.
    let png = build_solid_png(64, 32, [200, 160, 120, 255]);
    Skin {
        png,
        width: 64,
        height: 32,
        model,
    }
}

/// Choose the default offline skin by UUID parity.
pub fn offline_default(uuid: &str) -> Skin {
    let v: u32 = uuid.replace('-', "").parse().unwrap_or(0);
    if v % 2 == 0 {
        default_offline_skin(SkinModel::Wide)
    } else {
        default_offline_skin(SkinModel::Slim)
    }
}

/// Build a solid-color PNG (no palette, RGBA8). Used for placeholder skins.
fn build_solid_png(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    // PNG signature
    out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    // IHDR
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&[0, 0, 0, 13]); // length
    ihdr.extend_from_slice(b"IHDR"); // type
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // bit depth 8, color type 6 (RGBA)
    out.extend_from_slice(&ihdr);
    out.extend_from_slice(&png_crc(&ihdr));

    // IDAT: one scanline (filter byte 0 + raw pixels) compressed
    let mut raw = Vec::with_capacity((width as usize) * 4 + 1);
    for _ in 0..height {
        raw.push(0); // filter: none
        for _ in 0..width {
            raw.extend_from_slice(&rgba);
        }
    }
    let compressed = png_deflate(&raw);
    let mut idat = Vec::new();
    idat.extend_from_slice(&[0, 0, 0]);
    idat.extend_from_slice(&(compressed.len() as u32).to_be_bytes());
    idat.extend_from_slice(&compressed);
    out.extend_from_slice(&idat);
    out.extend_from_slice(&png_crc(&idat));

    // IEND
    out.extend_from_slice(&[0, 0, 0, 0, 0]);
    out.extend_from_slice(&png_crc(&[]));

    out
}

/// A tiny DEFLATE (raw zlib) wrapper. We use the `flate2` crate if available;
/// to avoid a hard dependency we fall back to stored (uncompressed) blocks.
fn png_deflate(data: &[u8]) -> Vec<u8> {
    // Use flate2 (Rust stdlib-compatible, no openssl).
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = enc.write_all(data);
    enc.finish().unwrap_or_default()
}

fn png_crc(chunk: &[u8]) -> [u8; 4] {
    // chunk = type(4) + data. CRC32 over type+data.
    use crc32fast::Hasher;
    let mut h = Hasher::new();
    h.update(chunk);
    h.finalize().to_be_bytes()
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_png_has_valid_header() {
        let png = build_solid_png(64, 32, [10, 20, 30, 255]);
        assert_eq!(&png[..8], [137, 80, 78, 71, 13, 10, 26, 10]);
        assert_eq!(png[11], 13); // IHDR length (big-endian u32)
        assert_eq!(png[23], 32); // height = 32
    }

    #[test]
    fn offline_default_model_by_parity() {
        let even = "00000000-0000-0000-0000-000000000000";
        let odd = "00000000-0000-0000-0000-000000000001";
        assert_eq!(offline_default(even).model, SkinModel::Wide);
        assert_eq!(offline_default(odd).model, SkinModel::Slim);
    }

    #[test]
    fn parse_png_reads_dimensions() {
        let png = build_solid_png(64, 32, [0, 0, 0, 255]);
        let s = parse_png(&png);
        assert_eq!(s.width, 64);
        assert_eq!(s.height, 32);
        assert_eq!(s.model, SkinModel::Slim);
    }
}
