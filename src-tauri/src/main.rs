// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod helper;
mod version;

use std::env;
use std::fs::File;
use std::io::BufRead;
use std::path::PathBuf;

use anyhow::anyhow;
use tauri::api::dialog::blocking::MessageDialogBuilder;
use tauri::api::dialog::MessageDialogButtons;
use tauri::api::dialog::MessageDialogKind;
use tauri::{CustomMenuItem, Env, Manager, Menu, MenuItem, Submenu};

use crate::helper::{
    copy_and_exec, exec_cmd_bash_script, exec_cmd_script, exec_script, is_wsl_running, try_url,
};
use crate::version::{get_latest_version, parse_version, show_version};
use error::Error;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use rev_buf_reader::RevBufReader;
use sentry::protocol::{Attachment, AttachmentType};
use sentry::Scope;
use tauri::api::dialog;
use tauri::api::path::{resolve_path, BaseDirectory};
use tauri::async_runtime::block_on;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

const SENTRY_DSN: &'static str = match option_env!("SENTRY_DSN") {
    Some(v) => v,
    None => "https://763b074178d15f7cbc24cad779fd00ac@o4504610754265088.ingest.us.sentry.io/4507299346120704",
};

#[tauri::command]
async fn check_install(handle: tauri::AppHandle) -> Result<String, Error> {
    tracing::info!("handle check_install, url: {}", handle.get_window("main").unwrap().url());

    let menu = handle.get_window("main").unwrap().menu_handle();
    menu.get_item("stop").set_enabled(false)?;
    menu.get_item("uninstall").set_enabled(false)?;
    handle.emit_all("status", String::from("Checking components..."))?;

    match env::consts::OS {
        "macos" | "linux" => {
            let binary_path = match find_sonaric_binary() {
                Some(p) => p,
                None => return Ok("install".to_string()),
            };
            let binary_path_str = binary_path.to_str().ok_or(anyhow!("Invalid binary path"))?;
            tracing::info!("Sonaric binary found: {}", binary_path_str);
            menu.get_item("uninstall").set_enabled(true)?;

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
                        if latest_version.gt(&version) {
                            tracing::info!("Update available: {} -> {}", version, latest_version);
                            return Ok("update".to_string());
                        }
                    }

                    if res.contains("daemon is not running") {
                        // sonaric is installed but not running
                        return Ok("start".to_string());
                    }
                }
                Err(e) => return Err(Error::from(e)),
            }
        }
        "windows" => {
            let res = exec_cmd_script(vec!["/C", "wsl", "--version"]).await?;
            if !res.success {
                tracing::debug!("WSL is not installed");
                return Ok("install".to_string());
            }

            let re = Regex::new(r"^.+ 2\..+$").unwrap();
            let wsl2 = match res.stdout.lines().next() {
                Some(line) => re.is_match(line),
                None => false,
            };
            if !wsl2 {
                tracing::debug!("WSL2 is not installed {}", res.stdout);
                return Ok("install".to_string());
            }

            let res = exec_cmd_script(vec!["/C", "wsl", "--list"]).await?;
            if !res.success || !res.stdout.contains("Ubuntu-22.04") {
                tracing::debug!("Ubuntu-22.04 is not installed");
                return Ok("install".to_string());
            }

            let res = exec_cmd_script(vec!["/C", "wsl", "--list", "--running"]).await?;
            if !res.success || !res.stdout.contains("Ubuntu-22.04") {
                tracing::debug!("Ubuntu-22.04 is not running");
                return Ok("start".to_string());
            }

            menu.get_item("uninstall").set_enabled(true)?;

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
            if res.stderr.contains("daemon is not running") {
                tracing::debug!("Sonaric daemon is not running");
                return Ok("start".to_string());
            }
            if !res.stdout.contains("version") {
                tracing::debug!("Sonaic is not installed");
                return Ok("install".to_string());
            }
            let version = parse_version(res.stdout.clone())?;
            let latest_version = get_latest_version().await?;
            if latest_version.gt(&version) {
                tracing::info!("Update available: {} -> {}", version, latest_version);
                return Ok("update".to_string());
            }
        }
        _ => return Err(Error::from(anyhow!("Unsupported OS"))),
    }

    Ok(check_gui(handle).await.unwrap_or_else(|e| {
        tracing::warn!("{}", e);
        "start".to_string()
    }))
}

fn find_sonaric_binary() -> Option<PathBuf> {
    let p = env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .filter_map(|dir| {
                let full_path = dir.join("sonaric");
                if full_path.is_file() {
                    Some(full_path)
                } else {
                    None
                }
            })
            .next()
    });

    if p == None {
        let paths = vec![
            "/usr/local/bin/sonaric",
            "/usr/bin/sonaric",
            "/opt/homebrew/bin/sonaric",
        ];
        for path in paths {
            if PathBuf::from(path).is_file() {
                return Some(PathBuf::from(path));
            }
        }
    }

    p
}

#[tauri::command]
async fn check_gui(handle: tauri::AppHandle) -> Result<String, Error> {
    tracing::info!("handle check_gui");

    let menu = handle.get_window("main").unwrap().menu_handle();
    handle.emit_all("status", String::from("Checking GUI..."))?;

    match try_url("http://localhost:44004").await {
        Ok(body) => {
            if body.contains("Sonaric") {
                menu.get_item("uninstall").set_enabled(true)?;
                menu.get_item("stop").set_enabled(true)?;
                Ok("OK".to_string())
            } else {
                Err(Error::from(anyhow!(format!(
                    "Unexpected response: {}",
                    body
                ))))
            }
        }
        Err(e) => {
            tracing::warn!("{}", e);
            Err(Error::RetryError("GUI is not available")) // retry
        }
    }
}

#[tauri::command]
async fn install_deps(handle: tauri::AppHandle) -> Result<String, Error> {
    tracing::info!("handle install_deps");
    handle.emit_all("status", String::from("Installing dependencies..."))?;

    match env::consts::OS {
        "macos" => install_deps_mac(handle).await,
        "windows" => install_deps_win(handle).await,
        "linux" => install_deps_linux(handle).await,
        _ => Err(Error::from(anyhow!("Unsupported OS"))),
    }
}

async fn install_deps_win(handle: tauri::AppHandle) -> Result<String, Error> {
    let res = exec_cmd_script(vec!["/C", "wsl", "--version"]).await?;
    if !res.success {
        tracing::debug!("WSL is not installed");
        return Err(Error::from(anyhow!("It looks like WSL is not installed. Please install WSL from Microsoft Store (https://aka.ms/wslstorepage) and try again.")));
    }

    let re = Regex::new(r"^.+ 2\..+$").unwrap();
    let wsl2 = match res.stdout.lines().next() {
        Some(line) => re.is_match(line),
        None => false,
    };
    if !wsl2 {
        tracing::debug!("WSL2 is not installed {}", res.stdout);
        return Err(Error::from(anyhow!("It looks like you are using WSL 1. Please upgrade to WSL 2 (https://aka.ms/wslstorepage) and try again.")));
    }

    let resource_path = get_resource_path(handle.clone(), "res/install-win.bat")?;

    exec_script(
        handle,
        "cmd",
        vec!["/C", resource_path.as_str()],
        true,
        true,
    )
    .await
}

async fn install_deps_linux(handle: tauri::AppHandle) -> Result<String, Error> {
    let resource_path = get_resource_path(handle.clone(), "res/install-linux.sh")?;
    let tmp_path = env::temp_dir().join("sonaric-install.sh");
    let tmp_path_str = tmp_path.to_str().ok_or(anyhow!("Invalid temp path"))?;

    let appimage_path = env::var("APPIMAGE").unwrap_or("".to_string());
    if !appimage_path.is_empty() {
        match create_desktop_entry(handle.clone(), appimage_path) {
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("create desktop entry: {}", e);
            }
        }
    }

    copy_and_exec(handle, resource_path.as_str(), tmp_path_str).await
}

fn create_desktop_entry(handle: tauri::AppHandle, appimage_path: String) -> Result<(), Error> {
    let full_path = PathBuf::from(format!(
        "{}/.local/share/applications/sonaric.desktop",
        env::var("HOME").unwrap_or("".to_string()),
    ));

    // skip if present or application folder does not exist
    if full_path.exists() || !full_path.parent().unwrap().exists() {
        return Ok(());
    }

    let tmp_icon_res = get_resource_path(handle.clone(), "res/icon.png")?;

    // copy icon to user's home directory
    let icon_res_path = handle
        .path_resolver()
        .app_config_dir()
        .ok_or(anyhow!("Invalid config path"))?;

    let icon_res_path = icon_res_path.join("icon.png");
    std::fs::copy(tmp_icon_res.as_str(), icon_res_path.as_os_str())?;

    let icon_res = icon_res_path.to_str().ok_or(anyhow!("Invalid icon path"))?;

    let content = format!(
        "[Desktop Entry]
Name=Sonaric
Exec={}
Icon={}
Type=Application
Categories=",
        appimage_path, icon_res
    );

    std::fs::write(full_path, content)?;

    Ok(())
}

async fn install_deps_mac(handle: tauri::AppHandle) -> Result<String, Error> {
    let resource_path = get_resource_path(handle.clone(), "res/install-mac.sh")?;

    exec_script(handle, "bash", vec![resource_path.as_str()], true, true).await
}

#[tauri::command]
async fn stop_daemon(handle: tauri::AppHandle) -> Result<String, Error> {
    tracing::info!("handle stop_daemon");
    handle.emit_all("status", String::from("Stopping..."))?;

    match env::consts::OS {
        "macos" => stop_daemon_mac(handle).await,
        "windows" => stop_daemon_win(handle).await,
        "linux" => stop_daemon_linux(handle).await,
        _ => Err(Error::from(anyhow!("Unsupported OS"))),
    }
}

async fn stop_daemon_mac(handle: tauri::AppHandle) -> Result<String, Error> {
    let resource_path = get_resource_path(handle.clone(), "res/stop-mac.sh")?;

    exec_script(handle, "bash", vec![resource_path.as_str()], true, true).await
}

async fn stop_daemon_linux(handle: tauri::AppHandle) -> Result<String, Error> {
    let resource_path = get_resource_path(handle.clone(), "res/stop-linux.sh")?;
    let tmp_path = env::temp_dir().join("sonaric-stop.sh");
    let tmp_path_str = tmp_path.to_str().ok_or(anyhow!("Invalid temp path"))?;

    copy_and_exec(handle, resource_path.as_str(), tmp_path_str).await
}

async fn stop_daemon_win(handle: tauri::AppHandle) -> Result<String, Error> {
    let res = exec_cmd_script(vec!["/C", "wsl", "--list", "--running"]).await?;
    if !res.success || !res.stdout.contains("Ubuntu-22.04") {
        return Ok("WSL distribution is not running".to_string());
    }

    let resource_path = get_resource_path(handle.clone(), "res/stop-win.bat")?;

    exec_script(
        handle,
        "cmd",
        vec!["/C", resource_path.as_str()],
        true,
        true,
    )
    .await?;

    let confirmation = "Terminate Ubuntu-22.04 distribution in WSL?".to_string();
    let confirmed = MessageDialogBuilder::new("WSL confirmation", confirmation)
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::YesNo)
        .show();
    if confirmed {
        let res = exec_cmd_script(vec!["/C", "wsl", "--terminate", "Ubuntu-22.04"]).await?;
        if !res.success {
            tracing::error!("{}", res.stderr);
            return Err(Error::from(anyhow!("Failed to terminate WSL distribution")));
        }
    }

    Ok("Successfully stopped".to_string())
}

#[tauri::command]
async fn uninstall_daemon(handle: tauri::AppHandle) -> Result<String, Error> {
    tracing::info!("handle uninstall_daemon");
    handle.emit_all("status", String::from("Removing dependencies..."))?;

    match env::consts::OS {
        "macos" => uninstall_daemon_mac(handle).await,
        "windows" => uninstall_daemon_win(handle).await,
        "linux" => uninstall_daemon_linux(handle).await,
        _ => Err(Error::from(anyhow!("Unsupported OS"))),
    }
}

async fn uninstall_daemon_mac(handle: tauri::AppHandle) -> Result<String, Error> {
    let resource_path = get_resource_path(handle.clone(), "res/uninstall-mac.sh")?;

    exec_script(handle, "bash", vec![resource_path.as_str()], true, true).await
}

async fn uninstall_daemon_linux(handle: tauri::AppHandle) -> Result<String, Error> {
    let resource_path = get_resource_path(handle.clone(), "res/uninstall-linux.sh")?;
    let tmp_path = env::temp_dir().join("sonaric-remove.sh");
    let tmp_path_str = tmp_path.to_str().ok_or(anyhow!("Invalid temp path"))?;

    let appimage_path = env::var("APPIMAGE").unwrap_or("".to_string());
    let home_path = env::var("HOME").unwrap_or("".to_string());
    if !appimage_path.is_empty() && !home_path.is_empty() {
        // remove desktop shortcut
        let full_path = PathBuf::from(format!(
            "{}/.local/share/applications/sonaric.desktop",
            home_path
        ));

        if full_path.exists() {
            match std::fs::remove_file(full_path) {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("remove desktop entry: {}", e);
                }
            }
        }
    }

    copy_and_exec(handle, resource_path.as_str(), tmp_path_str).await
}

async fn uninstall_daemon_win(handle: tauri::AppHandle) -> Result<String, Error> {
    let res = exec_cmd_script(vec!["/C", "wsl", "--list"]).await?;
    if !res.success || !res.stdout.contains("Ubuntu-22.04") {
        return Ok("WSL distribution is not installed".to_string());
    }

    let resource_path = get_resource_path(handle.clone(), "res/uninstall-win.bat")?;

    exec_script(
        handle,
        "cmd",
        vec!["/C", resource_path.as_str()],
        true,
        true,
    )
    .await?;

    let confirmation = "Unregister Ubuntu-22.04 distribution from WSL?
Caution: Once unregistered, all data, settings, and software associated with that distribution will be permanently lost".to_string();
    let confirmed = MessageDialogBuilder::new("WSL confirmation", confirmation)
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::YesNo)
        .show();
    if confirmed {
        let res = exec_cmd_script(vec!["/C", "wsl", "--unregister", "Ubuntu-22.04"]).await?;
        if !res.success {
            tracing::error!("{}", res.stderr);
            return Err(Error::from(anyhow!(
                "Failed to unregister WSL distribution"
            )));
        }
    }
    Ok("Successfully uninstalled".to_string())
}

struct BaseUrl(String);

fn get_resource_path(handle: tauri::AppHandle, res: &str) -> Result<String, Error> {
    let resource_path = handle
        .path_resolver()
        .resolve_resource(res)
        .ok_or(anyhow!("Invalid install path"))?;

    #[cfg(windows)]
    let resource_path = std::path::Path::new(
        resource_path
            .as_path()
            .to_string_lossy()
            .trim_start_matches(r"\\?\"),
    )
    .to_path_buf();

    let resource_path_str = resource_path
        .as_path()
        .to_str()
        .ok_or(anyhow!("Invalid resource path"))?;

    Ok(resource_path_str.to_string())
}

fn main() {
    let ctx = tauri::generate_context!();

    let log_path = resolve_path(
        ctx.config(),
        ctx.package_info(),
        &Env::default(),
        "app.log",
        Some(BaseDirectory::AppLog),
    )
    .expect("failed to resolve path");

    // Register the Sentry tracing layer to capture breadcrumbs, events, and spans:
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_filter(EnvFilter::from("debug,hyper=info")),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .compact()
                .with_writer(tracing_appender::rolling::never(
                    log_path.parent().unwrap(),
                    "app.log",
                ))
                .with_filter(EnvFilter::from("debug,hyper=info")),
        )
        .with(sentry_tracing::layer())
        .init();

    tracing::info!("log path: {:?}", log_path);
    let release = format!(
        "{} ({})",
        ctx.config()
            .package
            .version
            .clone()
            .unwrap_or("unknown".to_string())
            .to_string(),
        env!("GIT_VERSION")
    );
    tracing::info!("app version: {}", release);

    let _guard = sentry::init((
        SENTRY_DSN,
        sentry::ClientOptions {
            release: Some(release.into()),
            ..Default::default()
        },
    ));

    let menu = Menu::new()
        .add_submenu(Submenu::new(
            "Application",
            Menu::new()
                .add_item(CustomMenuItem::new("reload", "Reload").accelerator("CmdOrCtrl+R"))
                .add_native_item(MenuItem::Separator)
                .add_item(CustomMenuItem::new("stop", "Stop").disabled())
                .add_item(CustomMenuItem::new("uninstall", "Uninstall").disabled())
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Quit),
        ))
        .add_submenu(Submenu::new(
            "Help",
            Menu::new()
                .add_item(CustomMenuItem::new("about", "About"))
                .add_item(CustomMenuItem::new("docs", "Documentation")),
        ));

    tauri::Builder::default()
        .menu(menu)
        .on_page_load(|window, _payload| {
            window
                .app_handle()
                .manage(BaseUrl(window.url().to_string()));
        })
        .on_menu_event(|event| {
            tracing::info!("menu event: {:?}", event.menu_item_id());
            match event.menu_item_id() {
                "stop" => dialog::ask(
                    Some(&event.window().clone()),
                    "Stop Sonaric",
                    "Are you sure you want to stop Sonaric?",
                    move |answer| {
                        if answer {
                            let app = event.window().app_handle();
                            let base_url = app.state::<BaseUrl>();
                            let code = format!(
                                "window.location.href = '{}?action=stop'",
                                base_url.0.to_string(),
                            );
                            tracing::info!("code: {}", code);
                            event.window().unmaximize().unwrap();
                            event.window().eval(code.as_str()).unwrap();
                        }
                    },
                ),
                "uninstall" => dialog::ask(
                    Some(&event.window().clone()),
                    "Uninstall Sonaric",
                    "Are you sure you want to uninstall Sonaric?",
                    move |answer| {
                        if answer {
                            let app = event.window().app_handle();
                            let base_url = app.state::<BaseUrl>();
                            let code = format!(
                                "window.location.href = '{}?action=uninstall'",
                                base_url.0.to_string(),
                            );
                            tracing::info!("code: {}", code);
                            event.window().unmaximize().unwrap();
                            event.window().eval(code.as_str()).unwrap();
                        }
                    },
                ),
                "reload" => event.window().eval("window.location.reload()").unwrap(),
                "about" => {
                    let mes  = match block_on(show_version(event.window().app_handle().clone())) {
                        Ok(res) => format!("The Sonaric AI node can be deployed in one click and automates the deployment and management of any blockchain node.\n\n\
App version: {}\n\
Daemon version: {}\n\
GUI version: {}\n", res.app, res.daemon, res.gui),
                        Err(e) => {
                            tracing::error!("show version: {}", e);
                            "The Sonaric AI node can be deployed in one click and automates the deployment and management of any blockchain node.".to_string()
                        }
                    };

                    dialog::message(
                        Some(&event.window().clone()),
                        "Sonaric AI Node",
                        mes,
                    )
                },
                "docs" => tauri::api::shell::open(
                    &event.window().shell_scope(),
                    "https://docs.sonaric.xyz/".to_string(),
                    None,
                )
                .unwrap(),
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            install_deps,
            check_install,
            check_gui,
            stop_daemon,
            uninstall_daemon,
            show_version,
            report_bug,
        ])
        .run(ctx)
        .expect("error while running tauri application");
}

#[derive(Clone, serde::Serialize)]
pub struct FeedbackBody {
    pub name: String,
    pub email: String,
    pub comments: String,
    pub event_id: String,
}

#[tauri::command]
async fn report_bug(
    handle: tauri::AppHandle,
    name: String,
    description: String,
) -> Result<(), Error> {
    tracing::info!("handle report_bug");

    let log_path = handle
        .path_resolver()
        .app_log_dir()
        .unwrap_or_default()
        .join("app.log");

    let wsl = match env::consts::OS {
        "windows" => is_wsl_running().await,
        _ => false,
    };

    let handle = tokio::task::spawn_blocking(move || {
        sentry::with_scope(
            |scope| {
                add_file_attachment(scope, log_path, "app-log.txt".to_string());

                match env::consts::OS {
                    "macos" | "linux" => add_file_attachment(
                        scope,
                        PathBuf::from("/var/lib/sonaricd/log/sonaricd.log"),
                        "sonaricd-log.txt".to_string(),
                    ),
                    "windows" => {
                        if wsl {
                            add_file_attachment(
                                scope,
                                PathBuf::from("\\\\wsl.localhost\\Ubuntu-22.04\\var\\lib\\sonaricd\\log\\sonaricd.log"),
                                "sonaricd-log.txt".to_string(),
                            );
                        }
                    }
                    _ => {}
                }

                scope.set_tag("source", "bug-report");
            },
            || -> Result<(), Error> {
                let uuid = sentry::capture_message(
                    format!("Bug report by {}\n{}", name, description).as_str(),
                    sentry::Level::Warning,
                );
                tracing::info!("Bug report captured: {}", uuid);

                let mut headers = HeaderMap::new();
                headers.insert("Content-Type", HeaderValue::from_static("application/json"));
                headers.insert(
                    "Authorization",
                    HeaderValue::from_str(format!("DSN {}", SENTRY_DSN).as_str()).unwrap(),
                );

                let body = serde_json::to_string(&FeedbackBody {
                    name: name,
                    email: "none@example.com".to_string(),
                    comments: description,
                    event_id: uuid.simple().to_string(),
                })?;

                let client = reqwest::blocking::Client::new();
                let response = client
                    .post("https://sentry.io/api/0/projects/monkos/sonaric-app/user-feedback/")
                    .headers(headers)
                    .body(body)
                    .send()?;
                if !response.status().is_success() {
                    tracing::warn!(
                        "failed to submit user feedback: {}, body: {:?}",
                        response.status(),
                        response.text()
                    );
                }
                Ok(())
            },
        )
    });

    handle.await.expect("should spawn report")
}

fn add_file_attachment(scope: &mut Scope, path: PathBuf, name: String) {
    let mut last_lines = File::open(path).map(|f| tail(&f, 500)).unwrap_or_default();

    if !last_lines.is_empty() {
        last_lines.reverse();
        scope.add_attachment(Attachment {
            ty: Some(AttachmentType::Attachment),
            buffer: last_lines.join("\n").into_bytes(),
            filename: name,
            content_type: Some("text/plain".to_string()),
        });
    }
}

fn tail(file: &File, limit: usize) -> Vec<String> {
    let buf = RevBufReader::new(file);
    buf.lines()
        .take(limit)
        .map(|l| l.expect("Could not parse line"))
        .collect()
}
