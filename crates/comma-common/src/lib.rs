//! Shared helper functions used by both `comma-cli` and `comma-tui`.
//!
//! Pure logic that does not depend on a particular UI or CLI framework
//! belongs here so the two front-ends stay consistent.

mod config;
mod state;

pub use config::{CommonConfig, Config, TuiSection};
pub use state::{load_last_dongle, resolve_tui_dongle, state_dir, store_last_dongle};

use anyhow::Result;
use comma_connect_api::RouteFiles;

/// Resolve a route name that may be given as a full `dongle|timestamp` or
/// as just the timestamp part.
pub fn resolve_route_name(
    name: &str,
    explicit_dongle: Option<&str>,
    default_dongle: Option<&str>,
) -> Result<String> {
    if name.contains('|') {
        return Ok(name.to_string());
    }
    let dongle = explicit_dongle
        .or(default_dongle)
        .ok_or_else(|| anyhow::anyhow!("short route name requires a dongle (pass one, set default, or use COMMA_DONGLE)"))?;
    Ok(format!("{dongle}|{name}"))
}

/// File kinds that can be requested for upload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Road,
    WideRoad,
    Driver,
    Log,
}

impl FileKind {
    pub fn filename(self) -> &'static str {
        match self {
            FileKind::Road => "fcamera.hevc",
            FileKind::WideRoad => "ecamera.hevc",
            FileKind::Driver => "dcamera.hevc",
            FileKind::Log => "rlog.zst",
        }
    }

    pub fn display(self) -> &'static str {
        match self {
            FileKind::Road => "road camera",
            FileKind::WideRoad => "wide road camera",
            FileKind::Driver => "driver camera",
            FileKind::Log => "log",
        }
    }

    pub fn uploaded<'a>(self, files: &'a RouteFiles) -> &'a [String] {
        match self {
            FileKind::Road => &files.cameras,
            FileKind::WideRoad => &files.ecameras,
            FileKind::Driver => &files.dcameras,
            FileKind::Log => &files.logs,
        }
    }
}

/// Compute the list of device paths that still need to be uploaded for a
/// route, optionally limited to the first `limit` segments.
pub fn missing_route_paths(
    route_ts: &str,
    segment_count: i32,
    files: &RouteFiles,
    limit: Option<u32>,
) -> Vec<String> {
    let segments = match limit {
        Some(l) => segment_count.min(l as i32),
        None => segment_count,
    };
    let kinds = [
        FileKind::Road,
        FileKind::Driver,
        FileKind::WideRoad,
        FileKind::Log,
    ];
    let mut paths = Vec::new();
    for n in 0..segments {
        for kind in kinds {
            // Include only when the uploaded array is shorter than this segment
            // (arrays are ordered by segment number; length > n means uploaded).
            if (n as usize) < kind.uploaded(files).len() {
                continue;
            }
            paths.push(format!("{route_ts}--{n}/{}", kind.filename()));
        }
    }
    paths
}

/// Unix timestamp (seconds) of `days` days before `now_secs`.
pub fn cutoff_secs(now_secs: u64, days: u32) -> i64 {
    (now_secs as i64) - (days as i64) * 24 * 60 * 60
}

#[cfg(test)]
mod tests {
    use super::*;
    use comma_connect_api::RouteFiles;

    #[test]
    fn full_name_is_passed_through() {
        let name = "44379c05b64fd7e3|00000144--e88a1a5266";
        let resolved = resolve_route_name(name, None, None).unwrap();
        assert_eq!(resolved, name);
    }

    #[test]
    fn short_name_uses_explicit_dongle() {
        let resolved =
            resolve_route_name("00000144--e88a1a5266", Some("44379c05b64fd7e3"), None).unwrap();
        assert_eq!(resolved, "44379c05b64fd7e3|00000144--e88a1a5266");
    }

    #[test]
    fn short_name_falls_back_to_default_dongle() {
        let resolved =
            resolve_route_name("00000144--e88a1a5266", None, Some("44379c05b64fd7e3")).unwrap();
        assert_eq!(resolved, "44379c05b64fd7e3|00000144--e88a1a5266");
    }

    #[test]
    fn short_name_without_dongle_errors() {
        let err = resolve_route_name("00000144--e88a1a5266", None, None).unwrap_err();
        assert!(err.to_string().contains("dongle"));
    }

    #[test]
    fn route_paths_are_zipped_by_segment_and_skip_uploaded() {
        let files = RouteFiles {
            cameras: vec!["a".into(), "b".into()],
            dcameras: vec!["a".into()],
            ecameras: vec![],
            logs: vec!["a".into(), "b".into()],
            ..Default::default()
        };
        let paths = missing_route_paths("ts", 2, &files, None);
        assert_eq!(
            paths,
            vec![
                "ts--0/ecamera.hevc".to_string(),
                "ts--1/dcamera.hevc".to_string(),
                "ts--1/ecamera.hevc".to_string(),
            ]
        );
    }

    #[test]
    fn route_paths_respect_segment_limit() {
        let files = RouteFiles::default();
        let paths = missing_route_paths("ts", 5, &files, Some(1));
        assert_eq!(
            paths,
            vec![
                "ts--0/fcamera.hevc".to_string(),
                "ts--0/dcamera.hevc".to_string(),
                "ts--0/ecamera.hevc".to_string(),
                "ts--0/rlog.zst".to_string(),
            ]
        );
    }

    #[test]
    fn cutoff_is_days_ago_in_seconds() {
        let cutoff = cutoff_secs(1792022400, 14);
        assert_eq!(cutoff, 1792022400 - 14 * 24 * 60 * 60);
    }
}
