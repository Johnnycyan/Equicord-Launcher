// For compiling the modloader DLL:
pub use electron_hook::*;

pub mod constants;
pub mod discord;
pub mod updater;

// Library for the binaries to use:
#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::*;

use clap::Parser;
use discord::{DiscordBranch, DiscordPath};

#[derive(clap::Parser, Debug)]
struct Args {
    /// To use a local instance of the mod, pass the path to the mod entrypoint.
    ///
    /// e.g. `--local "C:\\Users\\megu\\equicord\\dist\\injector.js"`
    #[clap(short, long)]
    pub local: Option<String>,

    /// Optional launch arguments to pass to the Discord executable
    ///
    /// e.g. `-- --start-minimized --enable-blink-features=MiddleClickAutoscroll`
    #[clap(allow_hyphen_values = true, last = true)]
    pub launch_args: Vec<String>,
}

pub async fn launch(instance_id: &str, branch: DiscordBranch, display_name: &str) {
    std::env::set_var("DISABLE_UPDATER_AUTO_PATCHING", "true");

    let args = Args::parse();

    let Some(discord_dir) = discord::get_discord(branch) else {
        let title = format!("No {display_name} installation found!");
        let message = format!(
            "Equicord couldn't find your Discord installation.\n\
			Try reinstalling {display_name} and try again."
        );

        #[cfg(not(windows))]
        {
            use dialog::DialogBox as _;
            let _ = dialog::Message::new(message).title(title).show();
        }

        #[cfg(windows)]
        messagebox(&title, &message, MessageBoxIcon::Error);

        return;
    };

    let library_path = constants::get_library_path();

    let assets_dir = constants::asset_cache_dir().unwrap();

    // If `--local` is provided, use a local build. Otherwise, download assets.
    let mod_entrypoint = if let Some(local_path) = args.local {
        local_path
    } else {
        // We can usually attempt to run Discord even if the downloads fail...
        // TODO: Make this more robust. Maybe specific error reasons so we can determine if it's safe to continue.
        // TODO: Make this more robust. Maybe specific error reasons so we can determine if it's safe to continue.
        let _ = updater::download_assets().await;
        let _ = updater::download_open_asar().await;

        assets_dir
            .join(constants::MOD_ENTRYPOINT)
            .to_string_lossy()
            .replace("\\", "\\\\")
            .to_string()
    };

    let branch_name = match branch {
        DiscordBranch::Stable => "stable",
        DiscordBranch::PTB => "ptb",
        DiscordBranch::Canary => "canary",
        DiscordBranch::Development => "development",
    };

    let asar = electron_hook::asar::Asar::new()
        .with_id(instance_id)
        .with_mod_entrypoint(&mod_entrypoint)
        .with_template(include_str!("./require.js"))
        .with_wm_class(&format!("equicord-{branch_name}"))
        .create()
        .unwrap();

    let asar_path = asar.to_string_lossy().to_string();

    match discord_dir {
        DiscordPath::Filesystem(discord_exe) => {
            // Check if we have OpenAsar downloaded
            let open_asar_source = constants::asset_cache_dir()
                .unwrap()
                .join(constants::OPEN_ASAR_FILENAME);

            if open_asar_source.exists() {
                if let Some(parent) = discord_exe.parent() {
                    let resources_dir = parent.join("resources");
                    let app_asar = resources_dir.join("app.asar");
                    let backup_asar = resources_dir.join("_app.asar");

                    // If backup doesn't exist, create it by renaming app.asar
                    if !backup_asar.exists() && app_asar.exists() {
                        println!("[Equicord Launcher] Backing up original app.asar...");
                        let _ = std::fs::rename(&app_asar, &backup_asar);
                    }

                    // Copy OpenAsar to app.asar
                    // We only do this if we successfully created a backup or if a backup already exists
                    if backup_asar.exists() {
                         println!("[Equicord Launcher] Patching OpenAsar...");
                         let _ = std::fs::copy(&open_asar_source, &app_asar);
                    }
                }
            }

            let discord_dir = discord_exe.to_string_lossy().to_string();

            electron_hook::launch(
                &discord_dir,
                &library_path,
                &asar_path,
                args.launch_args,
                false,
            )
            .unwrap();
        }
        #[cfg(target_os = "linux")]
        DiscordPath::FlatpakId(id) => {
            electron_hook::launch_flatpak(&id, &library_path, &asar_path, args.launch_args, false)
                .unwrap();
        }
        #[cfg(not(target_os = "linux"))]
        DiscordPath::FlatpakId(_) => {
            panic!("Flatpak is only supported on Linux");
        }
    }
}


