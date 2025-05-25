mod commands;
mod structs;

use std::io::Cursor;

use anyhow::Context as _;
use structs::message::Message;
use image::{DynamicImage, ImageReader};
use serenity::all::{CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction};
use serenity::{all::CreateAttachment};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;
use mozjpeg::{Compress, ColorSpace};

struct Bot{
    guild_id: String
}

#[async_trait]
impl EventHandler for Bot {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!("Received command interaction: {:#?}", command);

            let response_content = match command.data.name.as_str() {
                "test" => {
                    Some(Message{
                        text: "Hello World!".to_string(),
                        attachment: None,
                    })
                }
                "compress" => {
                    Some(commands::compress::run(&command.data.options()).await)
                }
                command => {
                    Some(Message{
                        text: format!("Unknown command: {}", command),
                        attachment: None,
                    })
                },
            };

            if let Some(response_content) = response_content {
                let data:CreateInteractionResponseMessage;
                if let Some(attachment) = response_content.attachment {
                    data = CreateInteractionResponseMessage::new()
                        .content(response_content.text)
                        .add_file(attachment); 
                } else {
                    data = CreateInteractionResponseMessage::new()
                        .content(response_content.text);
                };

                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(self.guild_id.parse().unwrap());

        let _ = guild_id
            .set_commands(
                &ctx.http,
                vec![
                    CreateCommand::new("test").description("Just a test command for various purposes"),
                    commands::compress::register()
                ]
            )
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

    let bot = Bot {
        guild_id
    };

    let client = Client::builder(&token, GatewayIntents::empty())
        .event_handler(bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
