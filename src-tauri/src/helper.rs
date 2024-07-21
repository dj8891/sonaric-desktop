use crate::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tauri::Manager;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone)]
pub(crate) struct ScriptOutput {
    pub(crate) success: bool,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) async fn try_url(url: &str) -> Result<String, Error> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

pub(crate) async fn copy_and_exec(
    handle: tauri::AppHandle,
    src: &str,
    dest: &str,
) -> Result<String, Error> {
    std::fs::copy(src, dest)?;

    match exec_script(
        handle,
        "/usr/bin/pkexec",
        vec!["/usr/bin/sh", dest],
        true,
        true,
    )
    .await
    {
        Ok(output) => Ok(output),
        Err(e) => {
            // check error with causes if it countains code 126
            if format!("{:?}", e).contains("exited with code 126") {
                Err(Error::RetryError(
                    "permission denied: please try again with elevated permissions",
                ))
            } else {
                Err(e)
            }
        }
    }
}

pub(crate) async fn exec_script(
    handle: tauri::AppHandle,
    cmd: &str,
    args: Vec<&str>,
    emit_event: bool,
    check_status: bool,
) -> Result<String, Error> {
    let mut h = duct::cmd(cmd, args).stderr_to_stdout().stdout_capture();
    if !check_status {
        h = h.unchecked();
    }

    let reader = h.reader()?;

    let mut output = String::new();
    for line in BufReader::new(reader).lines() {
        let ln = match line {
            Ok(ln) => ln,
            Err(e) => {
                // get last 8 lines from output
                let mut last_output = output.lines().rev().take(8).collect::<Vec<&str>>();
                last_output.reverse();

                // append output to error
                return Err(Error::from(
                    anyhow::Error::new(e)
                        .context(output.clone())
                        .context(format!(
                            "command execution failed:\n\n{}",
                            last_output.join("\n"),
                        )),
                ));
            }
        };
        // print to console
        tracing::debug!("{}", ln.clone());
        if emit_event {
            // send to frontend
            handle.emit_all("install-output", ln.clone())?;
        }
        // append to output result
        output.push_str(format!("{}\n", ln).as_str());
    }
    Ok(output)
}

pub(crate) async fn exec_cmd_script(args: Vec<&str>) -> Result<ScriptOutput, Error> {
    let mut command = Command::new("cmd");
    command.args(args).stdout(Stdio::piped());

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let child = command.spawn()?;
    let output = child.wait_with_output()?;
    let u16s: Vec<u16> = output
        .stdout
        .chunks_exact(2)
        .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
        .collect();
    let stdout = match String::from_utf16(&u16s) {
        Ok(stdout) => stdout,
        Err(e) => return Err(Error::from(e)),
    };
    Ok(ScriptOutput {
        success: output.status.success(),
        stdout: stdout,
        stderr: String::new(),
    })
}

pub(crate) async fn exec_cmd_bash_script(args: Vec<&str>) -> Result<ScriptOutput, Error> {
    let mut command = Command::new("cmd");
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let child = command.spawn()?;
    let output = child.wait_with_output()?;
    let stdout = match String::from_utf8(output.stdout) {
        Ok(stdout) => stdout,
        Err(e) => return Err(Error::from(e)),
    };
    let stderr = match String::from_utf8(output.stderr) {
        Ok(stdout) => stdout,
        Err(e) => return Err(Error::from(e)),
    };
    Ok(ScriptOutput {
        success: output.status.success(),
        stdout: stdout,
        stderr: stderr,
    })
}

pub(crate) async fn is_wsl_running() -> bool {
    let installed = match exec_cmd_script(vec!["/C", "wsl", "--version"]).await {
        Ok(res) => res.success,
        _ => false,
    };
    if !installed {
        return false;
    }
    match exec_cmd_script(vec!["/C", "wsl", "--list", "--running"]).await {
        Ok(res) => res.success && res.stdout.contains("Ubuntu-22.04"),
        _ => false,
    }
}
