use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::env::VarError;
use std::io::{Cursor, Read};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    match get_release_assets().await {
        Ok(_) => {
            log::info!("Downloaded assets successfully");
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };
}

// create the error type that represents all errors possible in our program
#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    VarError(#[from] VarError),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Deserialize)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
struct Asset {
    name: String,
    url: String,
    browser_download_url: String,
}

async fn get_release_assets() -> Result<Vec<Asset>, Error> {
    let tag = std::env::var("GITHUB_TAG")?;
    let token = std::env::var("GITHUB_TOKEN")?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "X-GitHub-Api-Version",
        HeaderValue::from_static("2022-11-28"),
    );
    headers.insert("User-Agent", HeaderValue::from_static("sonaric-ci"));

    log::info!(
        "Fetching release information for tag: {}: {:?}",
        tag,
        headers
    );

    // The above command will return the release information

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "https://api.github.com/repos/monk-io/sonaric-desktop/releases/tags/{}",
            tag
        ))
        .headers(headers.clone())
        .bearer_auth(token.clone())
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(Error::Anyhow(anyhow::anyhow!(
            "Failed to fetch release information: {}, body: {:?}",
            response.status(),
            response.text().await
        )));
    }

    let resp_text = response.text().await?;
    let release: Release = serde_json::from_str(&resp_text)?;

    headers.insert(
        "Accept",
        HeaderValue::from_static("application/octet-stream"),
    );
    for asset in release.assets.iter() {
        log::info!(
            "Downloading: {}, url {}, headers: {:?}",
            asset.name,
            asset.browser_download_url,
            headers,
        );
        let response = client
            .get(asset.url.clone())
            .headers(headers.clone())
            .bearer_auth(token.clone())
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Error::Anyhow(anyhow::anyhow!(
                "Failed to fetch asset: {}, body: {:?}",
                response.status(),
                response.text().await
            )));
        }
        let mut file = std::fs::File::create(format!("./upload/{}/{}", tag, asset.name))?;
        let mut content = Cursor::new(response.bytes().await?);
        std::io::copy(&mut content, &mut file)?;

        // replace url in latest.json to point to the sonaric bucket
        if asset.name == "latest.json" {
            let mut latest_json = std::fs::File::open(format!("./upload/{}/{}", tag, asset.name))?;
            let mut latest_json_content = String::new();
            latest_json.read_to_string(&mut latest_json_content)?;
            let latest_json_content = latest_json_content.replace(
                "https://github.com/monk-io/sonaric-desktop/releases/download/",
                "https://storage.googleapis.com/sonaric-releases/desktop/",
            );
            std::fs::write(
                format!("./upload/{}/{}", tag, asset.name),
                latest_json_content,
            )?;
        }

        // Copy the same file as latest version
        let latest_name = asset
            .name
            .replace(tag.strip_prefix("app-v").unwrap(), "latest");

        std::fs::copy(
            format!("./upload/{}/{}", tag, asset.name),
            format!("./upload/{}", latest_name),
        )?;
    }

    Ok(release.assets)
}
