#![windows_subsystem = "windows"]

use equicord_launcher::discord::DiscordBranch;

static INSTANCE_ID: &str = "EquicordCanary";
static DISCORD_BRANCH: DiscordBranch = DiscordBranch::Canary;
static DISPLAY_NAME: &str = "Discord Canary";

#[tokio::main]
async fn main() {
    equicord_launcher::launch(INSTANCE_ID, DISCORD_BRANCH, DISPLAY_NAME).await;
}
