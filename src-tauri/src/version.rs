use crate::helper::{exec_cmd_bash_script, exec_script, try_url, is_wsl_running};
use crate::{find_sonaric_binary, Error};
use anyhow::anyhow;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fmt::{Display, Formatter};
use tokio::join;

const NA: &str = "n/a";

#[derive(Clone, serde::Serialize)]
pub struct VersionPayload {
    pub daemon: AppVersion,
    pub gui: AppVersion,
    pub app: AppVersion,
}

#[derive(Clone, serde::Serialize)]
pub struct AppVersion {
    pub version: String,
    pub latest: String,
    pub up_to_date: bool,
}

impl Display for AppVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.version == NA {
            return write!(f, "{}", self.version);
        }
        if self.up_to_date {
            write!(f, "{} (up to date)", self.version)
        } else {
            write!(f, "{} (latest: {})", self.version, self.latest)
        }
    }
}

impl Default for AppVersion {
    fn default() -> Self {
        Self {
            version: NA.to_string(),
            latest: NA.to_string(),
            up_to_date: true,
        }
    }
}

#[tauri::command]
pub async fn show_version(handle: tauri::AppHandle) -> Result<VersionPayload, Error> {
    tracing::info!("handle show_version");

    let (app_version, daemon_version, gui_version) = join!(
        get_app_version(handle.clone()),
        get_daemon_version(handle.clone()),
        get_gui_version(),
    );

    Ok(VersionPayload {
        app: app_version.unwrap_or_else(|e| {
            tracing::warn!("get app version: {}", e);
            AppVersion::default()
        }),
        daemon: daemon_version.unwrap_or_else(|e| {
            tracing::warn!("get daemon version: {}", e);
            AppVersion::default()
        }),
        gui: gui_version.unwrap_or_else(|e| {
            tracing::warn!("get gui version: {}", e);
            AppVersion::default()
        }),
    })
}

pub async fn get_app_version(handle: tauri::AppHandle) -> Result<AppVersion, Error> {
    let resp = handle.updater().check().await?;

    Ok(AppVersion {
        version: resp.current_version().to_string(),
        latest: resp.latest_version().to_string(),
        up_to_date: !resp.is_update_available(),
    })
}

pub async fn get_daemon_version(handle: tauri::AppHandle) -> Result<AppVersion, Error> {
    match env::consts::OS {
        "macos" | "linux" => {
            let binary_path = match find_sonaric_binary() {
                Some(p) => p,
                None => return Err(Error::from(anyhow!("Sonaric binary not found"))),
            };
            let binary_path_str = binary_path.to_str().ok_or(anyhow!("Invalid binary path"))?;

            match exec_script(
                handle.clone(),
                binary_path_str,
                vec!["version"],
                false,
                false,
            )
            .await
            {
                Ok(res) => {
                    if res.contains("version") {
                        // sonaric is installed, check version
                        let version = parse_version(res.clone())?;
                        let latest_version = get_latest_version().await?;

                        Ok(AppVersion {
                            version: version.to_string(),
                            latest: latest_version.to_string(),
                            up_to_date: !latest_version.gt(&version),
                        })
                    } else {
                        // sonaric is not installed
                        Err(Error::from(anyhow!("Sonaric is not installed")))
                    }
                }
                Err(e) => return Err(Error::from(e)),
            }
        }
        "windows" => {
            if !is_wsl_running().await {
                return Err(Error::from(anyhow!("WSL distribution is not running")));
            }

            let res = exec_cmd_bash_script(vec![
                "/C",
                "wsl",
                "--distribution",
                "Ubuntu-22.04",
                "--user",
                "root",
                "--exec",
                "/bin/bash",
                "-c",
                "sonaric version",
            ])
            .await?;
            if !res.stdout.contains("version") {
                return Err(Error::from(anyhow!("Sonaric is not installed")));
            }
            let version = parse_version(res.stdout.clone())?;
            let latest_version = get_latest_version().await?;

            Ok(AppVersion {
                version: version.to_string(),
                latest: latest_version.to_string(),
                up_to_date: !latest_version.gt(&version),
            })
        }
        _ => return Err(Error::from(anyhow!("Unsupported OS"))),
    }
}

#[derive(Deserialize, Debug)]
struct Tags {
    manifest: HashMap<String, Manifest>,
}

#[derive(Deserialize, Debug)]
struct Manifest {
    tag: Vec<String>,
}

pub async fn get_gui_version() -> Result<AppVersion, Error> {
    let url = "http://127.0.0.1:44005/version";
    let body = try_url(url).await?;
    let ver = Version::parse(body.trim().trim_start_matches("v"))?;

    // get latest version from https://us-central1-docker.pkg.dev/v2/sonaric-platform/sonaric-public/sonaric-gui/tags/list
    let latest_body = try_url(
        "https://us-central1-docker.pkg.dev/v2/sonaric-platform/sonaric-public/sonaric-gui/tags/list",
    ).await?;

    let tags: Tags = serde_json::from_str(&latest_body)?;
    for (_, manifest) in tags.manifest.iter() {
        if manifest.tag.contains(&"latest".to_string()) {
            // get the first tag that is not "latest"
            let latest = manifest
                .tag
                .iter()
                .find(|t| t.starts_with("v"))
                .ok_or(anyhow!("GUI release not found in registry"))?;
            let latest_ver = Version::parse(latest.trim().trim_start_matches("v"))?;
            return Ok(AppVersion {
                version: ver.to_string(),
                latest: latest_ver.to_string(),
                up_to_date: !latest_ver.gt(&ver),
            });
        }
    }
    Ok(AppVersion {
        version: ver.to_string(),
        latest: NA.to_string(),
        up_to_date: true,
    })
}

pub fn parse_version(text: String) -> Result<Version, Error> {
    let prefix = "CLI version:";
    let suffix = ", ";

    let start = text
        .find(prefix)
        .ok_or(anyhow!("Version start not found"))?;
    let end = text[start..]
        .find(suffix)
        .ok_or(anyhow!("Version end not found"))?;

    let ver = text[start + prefix.len()..start + end]
        .trim()
        .trim_start_matches("v");

    Version::parse(ver).map_err(Error::from)
}

pub async fn get_latest_version() -> Result<Version, Error> {
    let url = "https://storage.googleapis.com/sonaric-releases/stable/linux/latest-version";
    let body = try_url(url).await?;
    let ver = Version::parse(body.trim().trim_start_matches("v"))?;
    Ok(ver)
}
