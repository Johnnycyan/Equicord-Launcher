#![windows_subsystem = "windows"]

use equicord_launcher::discord::DiscordBranch;

static INSTANCE_ID: &str = "EquicordStable";
static DISCORD_BRANCH: DiscordBranch = DiscordBranch::Stable;
static DISPLAY_NAME: &str = "Discord Stable";

#[tokio::main]
async fn main() {
    equicord_launcher::launch(INSTANCE_ID, DISCORD_BRANCH, DISPLAY_NAME).await;
}
