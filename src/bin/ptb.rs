#![windows_subsystem = "windows"]

use equicord_launcher::discord::DiscordBranch;

static INSTANCE_ID: &str = "EquicordPTB";
static DISCORD_BRANCH: DiscordBranch = DiscordBranch::PTB;
static DISPLAY_NAME: &str = "Discord PTB";

#[tokio::main]
async fn main() {
    equicord_launcher::launch(INSTANCE_ID, DISCORD_BRANCH, DISPLAY_NAME).await;
}
