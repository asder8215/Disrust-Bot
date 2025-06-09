mod commands;
mod structs;

use anyhow::Context as _;
use serenity::all::{EditInteractionResponse, Interaction};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use structs::message::Message;
use tracing::info;

struct Bot {
    guild_id: String,
}

#[async_trait]
impl EventHandler for Bot {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!("Received command interaction: {:#?}", command);

            // Defer response of the message since commands like compress
            // Could take over 3 seconds for the initial response
            if let Err(why) = command.defer(&ctx.http).await {
                println!("Cannot respond to slash command: {why}");
                println!("From here")
            }

            let response_content = match command.data.name.as_str() {
                "compress" => Some(commands::compress::run(&command.data.options()).await),
                command => Some(Message {
                    text: format!("Unknown command: {}", command),
                    attachment: None,
                }),
            };

            // Use EditInteractionResponse to edit the deferred message
            // to the results
            if let Some(response_content) = response_content {
                let data: EditInteractionResponse;
                if let Some(attachment) = response_content.attachment {
                    data = EditInteractionResponse::new()
                        .content(response_content.text)
                        .new_attachment(attachment);
                } else {
                    data = EditInteractionResponse::new().content(response_content.text);
                };

                if let Err(why) = command.edit_response(&ctx.http, data).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(self.guild_id.parse().unwrap());

        let _ = guild_id
            .set_commands(&ctx.http, vec![commands::compress::register()])
            .await;
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    // let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let guild_id = secrets
        .get("GUILD_ID")
        .context("'GUILD_ID' was not found")?;

    let bot = Bot { guild_id };

    let client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
