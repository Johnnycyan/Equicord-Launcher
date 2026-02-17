use std::collections::HashMap;
use tinyjson::JsonValue;
use tokio::task::JoinSet;

use crate::constants;

static USER_AGENT: &str = concat!("EquicordLauncher/", env!("CARGO_PKG_VERSION"));

struct GithubRelease {
    pub tag_name: String,
    pub name: String,
    pub updated_at: String,
}

struct GithubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub async fn download_assets() -> Option<()> {
    let assets_dir = constants::asset_cache_dir().unwrap();
    let release_file = assets_dir.join(constants::RELEASE_INFO_FILE);

    // Get the current release.json if it exists.
    let current_version = if release_file.exists() {
        match std::fs::read_to_string(&release_file) {
            Ok(data) => {
                let json: JsonValue = match data.parse() {
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!("[Equicord Launcher] Failed to parse release.json: {e:?}");
                        return None;
                    }
                };
                let object: &HashMap<_, _> = json.get()?;

                let tag_name: &String = object.get("tag_name")?.get()?;
                let name: &String = object.get("name")?.get()?;
                let updated_at: String = object
                    .get("updated_at")
                    .and_then(|v| v.get::<String>())
                    .cloned()
                    .unwrap_or_default();

                Some(GithubRelease {
                    tag_name: tag_name.clone(),
                    name: name.clone(),
                    updated_at,
                })
            }
            Err(e) => {
                eprintln!("[Equicord Launcher] Failed to read release.json: {e}");
                None
            }
        }
    } else {
        None
    };

    // Get the latest release manifest from GitHub.
    println!("[Equicord Launcher] Checking for updates...");
    let response = ureq::get(constants::RELEASE_URL)
        .header("User-Agent", USER_AGENT)
        .call();

    let mut response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[Equicord Launcher] Failed to fetch release info from GitHub: {e}");
            eprintln!("[Equicord Launcher] This may be due to rate limiting (60 requests/hour for unauthenticated requests).");
            return None;
        }
    };

    let status = response.status();
    if status != 200 {
        eprintln!(
            "[Equicord Launcher] GitHub API returned non-200 status: {status} - updates may be rate-limited."
        );
        return None;
    }

    let body = match response.body_mut().read_to_string() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[Equicord Launcher] Failed to read response body: {e}");
            return None;
        }
    };

    let json: JsonValue = match body.parse() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[Equicord Launcher] Failed to parse GitHub API response: {e:?}");
            return None;
        }
    };
    let object: &HashMap<_, _> = json.get()?;

    let tag_name: &String = object.get("tag_name")?.get()?;
    let name: &String = object.get("name")?.get()?;
    let updated_at: &String = object.get("updated_at")?.get()?;

    // If the latest release has the same updated_at timestamp as our current one, don't bother downloading.
    // We use updated_at instead of tag_name/name because Equicord uses a rolling "latest" tag.
    if let Some(release) = current_version {
        if release.updated_at == *updated_at {
            println!("[Equicord Launcher] Already up to date (updated_at: {updated_at}).");
            return Some(());
        }
        println!(
            "[Equicord Launcher] Update detected: cached updated_at='{}' vs remote updated_at='{updated_at}'",
            release.updated_at
        );
    }

    println!("[Equicord Launcher] An update is available... Downloading...");

    // Loop over the assets and find the ones we want.
    let assets: &Vec<_> = object.get("assets")?.get()?;
    let assets: Vec<_> = assets
        .iter()
        .filter_map(|asset| {
            let asset: &HashMap<_, _> = asset.get()?;

            let name: &String = asset.get("name")?.get()?;
            let browser_download_url: &String = asset.get("browser_download_url")?.get()?;
            if constants::RELEASE_ASSETS.contains(&name.as_str()) {
                Some(GithubReleaseAsset {
                    name: name.clone(),
                    browser_download_url: browser_download_url.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    if assets.is_empty() {
        eprintln!("[Equicord Launcher] No matching release assets found in the GitHub release.");
        return None;
    }

    println!(
        "[Equicord Launcher] Downloading {} assets...",
        assets.len()
    );

    // Spawn all the download tasks simultaneously.
    let mut tasks = JoinSet::new();
    for asset in assets {
        let url = asset.browser_download_url.clone();
        let asset_name = asset.name.clone();

        tasks.spawn(async move {
            let response = ureq::get(&url)
                .header("User-Agent", USER_AGENT)
                .call();

            let mut response = match response {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!(
                        "[Equicord Launcher] Failed to download asset '{asset_name}': {e}"
                    );
                    return None;
                }
            };

            let body = match response.body_mut().read_to_vec() {
                Ok(b) => b,
                Err(e) => {
                    eprintln!(
                        "[Equicord Launcher] Failed to read asset '{asset_name}': {e}"
                    );
                    return None;
                }
            };

            println!("[Equicord Launcher] Downloaded '{asset_name}' ({} bytes)", body.len());
            Some((asset.name, body))
        });
    }

    // Wait for each task to finish and write them to disk.
    let mut all_succeeded = true;
    while let Some(resp) = tasks.join_next().await {
        match resp {
            Ok(Some((name, body))) => {
                let path = assets_dir.join(&name);
                if let Err(e) = std::fs::write(&path, body) {
                    eprintln!("[Equicord Launcher] Failed to write asset '{name}' to disk: {e}");
                    all_succeeded = false;
                }
            }
            Ok(None) => {
                all_succeeded = false;
            }
            Err(e) => {
                eprintln!("[Equicord Launcher] Asset download task panicked: {e}");
                all_succeeded = false;
            }
        }
    }

    if !all_succeeded {
        eprintln!("[Equicord Launcher] Some assets failed to download. Update may be incomplete.");
        // Still write the release info so we don't re-download the successful ones,
        // but return None to signal the failure.
    }

    // Write the new release.json to disk.
    let release_json = format!(
        "{{\n\
        \t\"tag_name\": \"{tag_name}\",\n\
        \t\"name\": \"{name}\",\n\
        \t\"updated_at\": \"{updated_at}\"\n\
		}}"
    );

    if let Err(e) = std::fs::write(&release_file, release_json) {
        eprintln!("[Equicord Launcher] Failed to write release.json: {e}");
        return None;
    }

    if all_succeeded {
        println!("[Equicord Launcher] Update complete.");
        Some(())
    } else {
        None
    }
}

pub async fn download_open_asar() -> Option<()> {
    let assets_dir = constants::asset_cache_dir().unwrap();
    let open_asar_path = assets_dir.join(constants::OPEN_ASAR_FILENAME);
    let release_file = assets_dir.join(constants::OPEN_ASAR_RELEASE_INFO_FILE);

    // Get the current open_asar_release.json if it exists.
    let current_version = if release_file.exists() {
        match std::fs::read_to_string(&release_file) {
            Ok(data) => {
                let json: JsonValue = match data.parse() {
                    Ok(j) => j,
                    Err(e) => {
                        eprintln!(
                            "[Equicord Launcher] Failed to parse open_asar_release.json: {e:?}"
                        );
                        return None;
                    }
                };
                let object: &HashMap<_, _> = json.get()?;

                let tag_name: &String = object.get("tag_name")?.get()?;
                let name: &String = object.get("name")?.get()?;
                let updated_at: String = object
                    .get("updated_at")
                    .and_then(|v| v.get::<String>())
                    .cloned()
                    .unwrap_or_default();

                Some(GithubRelease {
                    tag_name: tag_name.clone(),
                    name: name.clone(),
                    updated_at,
                })
            }
            Err(e) => {
                eprintln!("[Equicord Launcher] Failed to read open_asar_release.json: {e}");
                None
            }
        }
    } else {
        None
    };

    println!("[Equicord Launcher] Checking for OpenAsar updates...");

    let response = ureq::get(constants::OPEN_ASAR_URL)
        .header("User-Agent", USER_AGENT)
        .call();

    let mut response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[Equicord Launcher] Failed to fetch OpenAsar release info: {e}");
            eprintln!("[Equicord Launcher] This may be due to rate limiting (60 requests/hour for unauthenticated requests).");
            return None;
        }
    };

    let status = response.status();
    if status != 200 {
        eprintln!(
            "[Equicord Launcher] GitHub API returned non-200 status for OpenAsar: {status}"
        );
        return None;
    }

    let body = match response.body_mut().read_to_string() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[Equicord Launcher] Failed to read OpenAsar response body: {e}");
            return None;
        }
    };

    let json: JsonValue = match body.parse() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[Equicord Launcher] Failed to parse OpenAsar API response: {e:?}");
            return None;
        }
    };
    let object: &HashMap<_, _> = json.get()?;

    let tag_name: &String = object.get("tag_name")?.get()?;
    let name: &String = object.get("name")?.get()?;
    let updated_at: &String = object.get("updated_at")?.get()?;

    // If the latest release has the same updated_at timestamp as our current one, don't bother downloading.
    if let Some(release) = current_version {
        // If file also exists (double check), then return
        if release.updated_at == *updated_at && open_asar_path.exists() {
            println!("[Equicord Launcher] OpenAsar already up to date.");
            return Some(());
        }
    }

    println!("[Equicord Launcher] OpenAsar update available... Downloading...");

    let assets: &Vec<_> = object.get("assets")?.get()?;
    // OpenAsar releases usually have "app.asar"
    let asset = assets.iter().find_map(|asset| {
        let asset: &HashMap<_, _> = asset.get()?;
        let name: &String = asset.get("name")?.get()?;
        if name == "app.asar" {
            let url: &String = asset.get("browser_download_url")?.get()?;
            Some(url.clone())
        } else {
            None
        }
    });

    let Some(asset_url) = asset else {
        eprintln!("[Equicord Launcher] Could not find 'app.asar' asset in OpenAsar release.");
        return None;
    };

    let response = ureq::get(&asset_url)
        .header("User-Agent", USER_AGENT)
        .call();

    let mut response = match response {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[Equicord Launcher] Failed to download OpenAsar: {e}");
            return None;
        }
    };

    let body = match response.body_mut().read_to_vec() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[Equicord Launcher] Failed to read OpenAsar download: {e}");
            return None;
        }
    };

    println!(
        "[Equicord Launcher] Downloaded OpenAsar ({} bytes)",
        body.len()
    );

    if let Err(e) = std::fs::write(&open_asar_path, body) {
        eprintln!("[Equicord Launcher] Failed to write OpenAsar to disk: {e}");
        return None;
    }

    // Write the new open_asar_release.json to disk.
    let release_json = format!(
        "{{\n\
        \t\"tag_name\": \"{tag_name}\",\n\
        \t\"name\": \"{name}\",\n\
        \t\"updated_at\": \"{updated_at}\"\n\
		}}"
    );

    if let Err(e) = std::fs::write(&release_file, release_json) {
        eprintln!("[Equicord Launcher] Failed to write open_asar_release.json: {e}");
        return None;
    }

    println!("[Equicord Launcher] OpenAsar update complete.");
    Some(())
}
