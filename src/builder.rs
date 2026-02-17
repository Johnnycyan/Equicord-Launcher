//! Custom build pipeline for building Equicord with userplugins.
//!
//! Handles cloning/updating the Equicord repo, syncing userplugins,
//! and running the build process.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use tinyjson::JsonValue;

use crate::constants;

#[cfg(windows)]
use crate::progress::ProgressWindow;

const TOTAL_STEPS: u32 = 6;

/// Check that git, node, and pnpm are available on PATH.
fn check_prerequisites() -> Result<(), String> {
    let commands = [
        ("git", &["--version"][..]),
        ("node", &["--version"]),
        ("pnpm", &["--version"]),
    ];

    for (cmd, args) in &commands {
        match Command::new(cmd).args(*args).output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("[Equicord Launcher] Found {}: {}", cmd, version.trim());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!(
                    "'{}' was found but returned an error: {}",
                    cmd,
                    stderr.trim()
                ));
            }
            Err(_) => {
                return Err(format!(
                    "'{}' is not installed or not in PATH.\n\
                    The --custom flag requires git, Node.js (>=18), and pnpm.\n\
                    Please install them and try again.",
                    cmd
                ));
            }
        }
    }

    Ok(())
}

/// Clone the Equicord repo, or pull latest if it already exists.
fn clone_or_update_repo(repo_dir: &Path) -> Result<(), String> {
    if repo_dir.join(".git").exists() {
        println!("[Equicord Launcher] Updating Equicord repository...");

        // Fetch and reset to origin/main to handle force pushes on rolling releases
        let fetch = Command::new("git")
            .args(["fetch", "origin", "main"])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| format!("Failed to run git fetch: {e}"))?;

        if !fetch.status.success() {
            let stderr = String::from_utf8_lossy(&fetch.stderr);
            return Err(format!("git fetch failed: {stderr}"));
        }

        let reset = Command::new("git")
            .args(["reset", "--hard", "origin/main"])
            .current_dir(repo_dir)
            .output()
            .map_err(|e| format!("Failed to run git reset: {e}"))?;

        if !reset.status.success() {
            let stderr = String::from_utf8_lossy(&reset.stderr);
            return Err(format!("git reset failed: {stderr}"));
        }
    } else {
        println!("[Equicord Launcher] Cloning Equicord repository...");

        // Make sure parent directory exists
        if let Some(parent) = repo_dir.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }

        let clone = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--branch",
                "main",
                constants::EQUICORD_REPO_URL,
                &repo_dir.to_string_lossy(),
            ])
            .output()
            .map_err(|e| format!("Failed to run git clone: {e}"))?;

        if !clone.status.success() {
            let stderr = String::from_utf8_lossy(&clone.stderr);
            return Err(format!("git clone failed: {stderr}"));
        }
    }

    Ok(())
}

/// Get the current HEAD commit hash.
fn get_git_hash(repo_dir: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("Failed to get git hash: {e}"))?;

    if !output.status.success() {
        return Err("Failed to get git commit hash".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Compute a simple hash of the userplugins directory based on file names, sizes, and mtimes.
fn hash_directory(dir: &Path) -> Result<String, String> {
    if !dir.exists() {
        return Ok("empty".to_string());
    }

    let mut entries = Vec::new();
    collect_dir_entries(dir, dir, &mut entries)?;
    entries.sort();

    // Simple hash: concatenate all entries and use their combined string
    let combined = entries.join("|");
    // Use a simple hash — we don't need cryptographic strength here
    let hash = combined
        .bytes()
        .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));

    Ok(format!("{:016x}", hash))
}

fn collect_dir_entries(base: &Path, dir: &Path, entries: &mut Vec<String>) -> Result<(), String> {
    let read_dir =
        std::fs::read_dir(dir).map_err(|e| format!("Failed to read directory {:?}: {e}", dir))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        if path.is_dir() {
            collect_dir_entries(base, &path, entries)?;
        } else {
            let metadata = entry
                .metadata()
                .map_err(|e| format!("Failed to read metadata: {e}"))?;
            let size = metadata.len();
            let modified = metadata
                .modified()
                .map(|t| {
                    t.duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
                .unwrap_or(0);
            entries.push(format!("{}:{}:{}", relative, size, modified));
        }
    }

    Ok(())
}

/// Clear and copy userplugins into the repo's src/userplugins/ directory.
fn sync_userplugins(userplugins_src: &Path, repo_dir: &Path) -> Result<(), String> {
    let dest = repo_dir.join("src").join("userplugins");

    // Remove existing userplugins
    if dest.exists() {
        std::fs::remove_dir_all(&dest)
            .map_err(|e| format!("Failed to clear userplugins dir: {e}"))?;
    }

    std::fs::create_dir_all(&dest).map_err(|e| format!("Failed to create userplugins dir: {e}"))?;

    // Copy all files/dirs from source to dest
    copy_dir_recursive(userplugins_src, &dest)?;

    let count = std::fs::read_dir(&dest).map(|d| d.count()).unwrap_or(0);
    println!("[Equicord Launcher] Synced {count} userplugin(s).");

    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    let read_dir = std::fs::read_dir(src).map_err(|e| format!("Failed to read {:?}: {e}", src))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            std::fs::create_dir_all(&dest_path)
                .map_err(|e| format!("Failed to create dir {:?}: {e}", dest_path))?;
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Failed to copy {:?}: {e}", src_path))?;
        }
    }

    Ok(())
}

struct BuildState {
    git_commit: String,
    userplugins_hash: String,
}

/// Load build state from disk.
fn load_build_state(cache_dir: &Path) -> Option<BuildState> {
    let state_file = cache_dir.join(constants::CUSTOM_BUILD_STATE_FILE);
    let data = std::fs::read_to_string(&state_file).ok()?;
    let json: JsonValue = data.parse().ok()?;
    let object: &HashMap<_, _> = json.get()?;

    let git_commit: &String = object.get("git_commit")?.get()?;
    let userplugins_hash: &String = object.get("userplugins_hash")?.get()?;

    Some(BuildState {
        git_commit: git_commit.clone(),
        userplugins_hash: userplugins_hash.clone(),
    })
}

/// Save build state to disk.
fn save_build_state(
    cache_dir: &Path,
    git_commit: &str,
    userplugins_hash: &str,
) -> Result<(), String> {
    let state_file = cache_dir.join(constants::CUSTOM_BUILD_STATE_FILE);
    let json = format!(
        "{{\n\
        \t\"git_commit\": \"{git_commit}\",\n\
        \t\"userplugins_hash\": \"{userplugins_hash}\"\n\
        }}"
    );
    std::fs::write(&state_file, json).map_err(|e| format!("Failed to write build state: {e}"))
}

/// Run pnpm install in the repo directory.
fn run_pnpm_install(repo_dir: &Path) -> Result<(), String> {
    println!("[Equicord Launcher] Running pnpm install...");

    let output = Command::new("pnpm")
        .args(["install", "--frozen-lockfile"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("Failed to run pnpm install: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "pnpm install failed:\nstdout: {stdout}\nstderr: {stderr}"
        ));
    }

    println!("[Equicord Launcher] pnpm install complete.");
    Ok(())
}

/// Run pnpm build in the repo directory.
fn run_pnpm_build(repo_dir: &Path) -> Result<(), String> {
    println!("[Equicord Launcher] Running pnpm build...");

    let output = Command::new("pnpm")
        .args(["build"])
        .current_dir(repo_dir)
        .output()
        .map_err(|e| format!("Failed to run pnpm build: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "pnpm build failed:\nstdout: {stdout}\nstderr: {stderr}"
        ));
    }

    println!("[Equicord Launcher] pnpm build complete.");
    Ok(())
}

/// Copy build output from dist/desktop/ to the asset cache directory.
fn copy_build_output(repo_dir: &Path, cache_dir: &Path) -> Result<(), String> {
    let dist_dir = repo_dir.join("dist").join("desktop");

    if !dist_dir.exists() {
        return Err(format!(
            "Build output directory does not exist: {:?}",
            dist_dir
        ));
    }

    for filename in constants::BUILD_OUTPUT_FILES {
        let src = dist_dir.join(filename);
        let dest = cache_dir.join(filename);

        if src.exists() {
            std::fs::copy(&src, &dest)
                .map_err(|e| format!("Failed to copy build output '{}': {e}", filename))?;
        } else {
            // Some files like .LEGAL.txt might not exist in fresh builds
            eprintln!(
                "[Equicord Launcher] Warning: build output '{}' not found, skipping.",
                filename
            );
        }
    }

    println!("[Equicord Launcher] Build output copied to cache.");
    Ok(())
}

/// Run the full custom build pipeline.
///
/// Returns `Ok(())` on success, `Err(message)` on failure.
pub fn run_custom_build(userplugins_dir: &str) -> Result<(), String> {
    let userplugins_path = PathBuf::from(userplugins_dir);

    if !userplugins_path.exists() {
        return Err(format!(
            "Userplugins directory does not exist: {}",
            userplugins_dir
        ));
    }

    if !userplugins_path.is_dir() {
        return Err(format!(
            "Userplugins path is not a directory: {}",
            userplugins_dir
        ));
    }

    let repo_dir = constants::equicord_repo_dir()
        .ok_or_else(|| "Failed to determine Equicord repo directory".to_string())?;
    let cache_dir = constants::asset_cache_dir()
        .ok_or_else(|| "Failed to determine asset cache directory".to_string())?;

    // Step 0: Check prerequisites (before showing progress window)
    println!("[Equicord Launcher] Checking prerequisites...");
    check_prerequisites()?;

    // Show progress window
    #[cfg(windows)]
    let progress = ProgressWindow::new("Building Equicord with userplugins...", TOTAL_STEPS);

    #[cfg(windows)]
    progress.update(1, "Updating Equicord repository...");

    // Step 1: Clone or update repo
    clone_or_update_repo(&repo_dir)?;
    let git_hash = get_git_hash(&repo_dir)?;
    println!("[Equicord Launcher] Current commit: {git_hash}");

    #[cfg(windows)]
    progress.update(2, "Syncing userplugins...");

    // Step 2: Sync userplugins
    sync_userplugins(&userplugins_path, &repo_dir)?;
    let plugins_hash = hash_directory(&userplugins_path)?;

    // Step 3: Check if rebuild is needed
    let needs_rebuild = match load_build_state(&cache_dir) {
        Some(state) => {
            if state.git_commit == git_hash && state.userplugins_hash == plugins_hash {
                // Also verify the output files actually exist
                let all_exist = constants::BUILD_OUTPUT_FILES
                    .iter()
                    .filter(|f| !f.contains("LEGAL"))
                    .all(|f| cache_dir.join(f).exists());

                if all_exist {
                    println!("[Equicord Launcher] Build is up to date, skipping rebuild.");
                    false
                } else {
                    println!("[Equicord Launcher] Build output missing, rebuilding...");
                    true
                }
            } else {
                println!(
                    "[Equicord Launcher] Changes detected (git: {} -> {}, plugins hash changed: {}), rebuilding...",
                    state.git_commit,
                    git_hash,
                    state.userplugins_hash != plugins_hash
                );
                true
            }
        }
        None => {
            println!("[Equicord Launcher] No previous build state found, building...");
            true
        }
    };

    if needs_rebuild {
        #[cfg(windows)]
        progress.update(3, "Installing dependencies (pnpm install)...");

        // Step 4: pnpm install
        run_pnpm_install(&repo_dir)?;

        #[cfg(windows)]
        progress.update(4, "Building Equicord (pnpm build)...");

        // Step 5: pnpm build
        run_pnpm_build(&repo_dir)?;

        #[cfg(windows)]
        progress.update(5, "Copying build output...");

        // Step 6: Copy output
        copy_build_output(&repo_dir, &cache_dir)?;

        // Save build state
        save_build_state(&cache_dir, &git_hash, &plugins_hash)?;
    }

    #[cfg(windows)]
    {
        progress.update(TOTAL_STEPS, "Build complete!");
        // Brief pause so user can see "Build complete!"
        std::thread::sleep(std::time::Duration::from_millis(500));
        progress.close();
    }

    println!("[Equicord Launcher] Custom build pipeline complete.");
    Ok(())
}
