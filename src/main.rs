use std::io::Cursor;

use anyhow::Context as _;
use image::{DynamicImage, ImageReader};
use serenity::all::{CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction};
use serenity::{all::CreateAttachment, async_trait};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;
use mozjpeg;

struct Bot{
    guild_id: String
}

// struct Bot;

/// Load image from Vec<u8> into a DynamicImage
fn load_image_from_bytes(pixels: Vec<u8>) -> Result<DynamicImage, Box<dyn std::error::Error + Send>> {
    let cursor = Cursor::new(pixels);

    let image = ImageReader::new(cursor)
        .with_guessed_format()
        .expect("Could not guess image format")
        .decode()
        .expect("Could not decode image");

    Ok(image)
}

/// Convert image to an rgb8 format
fn convert_to_rgb(image: DynamicImage) -> Vec<u8> {
    image.to_rgb8().into_raw()
}

/// Optimize image to jpeg, the quality 60-80 are recommended.
/// 
/// Adapted from: https://github.com/vicanso/imageoptimize/blob/670bd3f40851c87d5fea5bb3e0e841d98a404c60/src/images.rs#L285
async fn to_mozjpeg(width: usize, height: usize, pixels: Vec<u8>, quality: u8) -> Result<Vec<u8>, Box<dyn std::error::Error + Send>> {

    let image = match load_image_from_bytes(pixels.clone()) {
        Ok(loaded_img) => {
            // loaded_img
            let rgb_data = convert_to_rgb(loaded_img);
            let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
            comp.set_size(width, height);
            comp.set_quality(quality as f32);
            let mut comp = comp.start_compress(Vec::new()).expect("Data did not start compressing");
            comp.write_scanlines(&rgb_data).expect("Scanlines were not written");
            let data = comp.finish().expect("Data did not finish compressing");
            Ok(data)
        }
        Err(e) => {
            // Log the error or return it for higher-level handling
            eprintln!("Problem loading bytes: {e}");
            Err(e)
        }
    };
    image
}

#[async_trait]
impl EventHandler for Bot {

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info!("Received command interaction: {:#?}", command);

            // let user_id: i64 = command.user.id.into();

            let response_content = match command.data.name.as_str() {
                "test" => {
                    let data = CreateInteractionResponseMessage::new()
                        .content(
                            format!(
                                "Hello World!"
                            )
                        );
                    let builder = CreateInteractionResponse::Message(data);
                    
                    builder
                }
                "compress" => {
                    let argument = command
                        .data
                        .resolved
                        .attachments
                        .iter()
                        .next();

                    let response_val:CreateInteractionResponse = match argument {
                        Some((_ , attachment)) => {
                            // Ensure the attachment is an image
                            if let (Some(img_width), Some(img_height)) = (attachment.width, attachment.height) {
                                    // Download image data
                                    let filename = &attachment.filename;
                                    let size = attachment.size;
                                    let img_bytes = attachment.download().await.expect("Image could not be downloaded");

                                    // Attempt compressing image
                                    match to_mozjpeg(img_width as usize, img_height as usize, img_bytes.clone(), 70).await {
                                        Ok(compressed_data) => {
                                            // create compressed attachment from the compressed data returned by to_mozjpeg
                                            let compressed_attachment = CreateAttachment::bytes(compressed_data.clone(), filename);
                                            
                                            let data = if compressed_data.len() < 8388608 { 
                                                CreateInteractionResponseMessage::new()
                                                .content(format!(
                                                    "Reduced image size of {} from {} bytes to {} bytes",
                                                    filename,
                                                    size,
                                                    compressed_data.len()
                                                ))
                                                .add_file(compressed_attachment)
                                            } else {
                                                CreateInteractionResponseMessage::new()
                                                .content(format!(
                                                    "Could not compress image to <= 8 MB as per Discord's upload file limit"
                                                ))
                                            };

                                            let builder = CreateInteractionResponse::Message(data);
                                            
                                            builder
                                        }
                                        Err(e) => {
                                            let data = CreateInteractionResponseMessage::new()
                                                .content(
                                                    format!(
                                                        "Failed to compress image: {}", e
                                                    )
                                                );
                                            let builder = CreateInteractionResponse::Message(data);
                                            
                                            builder
                                            
                                        }
                                    }
                            } else {
                                let data = CreateInteractionResponseMessage::new()
                                    .content(
                                        format!(
                                            "Image dimensions not found."
                                        )
                                    );
                                let builder = CreateInteractionResponse::Message(data);
                                
                                builder
                            }
                        },
                        None => {
                            let data = CreateInteractionResponseMessage::new()
                                .content(
                                    format!(
                                        "User did not include an attachment."
                                    )
                                );
                            let builder = CreateInteractionResponse::Message(data);
                            builder
                        }
                    };
                    response_val
                },
                command => {
                    let data = CreateInteractionResponseMessage::new()
                        .content(
                            format!(
                                "Unknown command: {}", command
                            )
                        );
                    let builder = CreateInteractionResponse::Message(data);
                    builder
                },
            };
            if let Err(why) = command.create_response(&ctx.http, response_content).await {
                println!("Cannot respond to slash command: {why}");
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
                    CreateCommand::new("compress")
                    .description("Takes an image from the user and outputs a compressed image if under 8 MB")
                    .add_option(
                        CreateCommandOption::new(
                            serenity::all::CommandOptionType::Attachment,
                            "image",
                            "Image to compress"
                        )
                        .required(true)
                    ),
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

    // Set gateway intents, which decides what events the bot will be notified about
    // let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // let client = Client::builder(&token, intents)
    //     .event_handler(Bot)
    //     .await
    //     .expect("Err creating client");

    Ok(client.into())
}
