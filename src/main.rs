use anyhow::Context as _;
use serenity::all::{CreateCommand, CreateCommandOption, CreateMessage, Interaction};
use serenity::{all::CreateAttachment, async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};

// struct Bot{
//     guild_id: String
// }


struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
        else if msg.content.starts_with("!test") {
            if let Some(image) = msg.attachments.get(0) {
                // Ensure the attachment is an image
                if let Some(img_width) = image.width {
                    if let Some(img_height) = image.height {
                        // Download image data
                        let response = reqwest::get(&image.url).await.expect("Failed to download image");
                        let img_bytes = response.bytes().await.expect("Failed to read image bytes");
                        
                        // Prepare image for compression
                        let img_to_compress = turbojpeg::Image {
                            pixels: &img_bytes.to_vec()[..],
                            width: img_width as usize,
                            pitch: img_width as usize * turbojpeg::PixelFormat::RGB.size(),
                            height: img_height as usize,
                            format: turbojpeg::PixelFormat::RGB,
                        };
        
                        // Compress image
                        let mut compressor = turbojpeg::Compressor::new().expect("Failed to create compressor");
                        compressor.set_quality(70).expect("Failed to set quality");
                        compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2).expect("Failed to set subsampling");
        
                        // let mut output_buf = turbojpeg::OutputBuf::new_owned();
                        let output_buf = compressor.compress_to_vec(img_to_compress).expect("Failed to compress image");
        
                        // Create attachment for compressed image
                        let attachment = CreateAttachment::bytes(&*output_buf, &image.filename);
        
                        // Send response message
                        let builder = CreateMessage::new()
                            .content(format!(
                                "Reduced image size of {} from {} bytes to {} bytes",
                                &image.filename,
                                image.size,
                                output_buf.len()
                            ))
                            .add_file(attachment);
                        let _ = msg.channel_id.send_message(&ctx.http, builder).await.expect("Failed to send message");
                    } else {
                        let _ = msg.channel_id.say(&ctx.http, "Image dimensions not found.").await;
                    }
                } else {
                    let _ = msg.channel_id.say(&ctx.http, "Attachment is not an image.").await;
                }
            } else {
                let _ = msg.channel_id.say(&ctx.http, "No attachments found.").await;
            }
        }
        
    //     } else if msg.content.starts_with("!test") {
    //         let image = &msg.attachments[0];
    //         let filename = &image.filename;
    //         let img_size = image.size;
    //         let img_width = image.width.expect("Image width not found");
    //         let img_height = image.height.expect("Image height not found");
            
    //         let attachment = CreateAttachment::url(&ctx.http, &image.url).await;
    //         // let builder = CreateAttachment::url(http, url)
    //         let attachment = attachment.expect("Attachment not provided");
            
    //         let img_to_compress = turbojpeg::Image{
    //             pixels: attachment.data.as_slice(),
    //             width: img_width as usize,
    //             pitch: (img_width as usize * turbojpeg::PixelFormat::RGB.size() * 2), 
    //             height: img_height as usize,
    //             format: turbojpeg::PixelFormat::RGB
    //         };
    //         let mut compressor = turbojpeg::Compressor::new().expect("Compressor should be made");
    //         let _ = compressor.set_quality(70);
    //         let _ = compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2);

    //         let mut output_buf = turbojpeg::OutputBuf::new_owned();

    //         compressor.compress(img_to_compress.as_deref(), &mut output_buf).expect("Could not compress image");

    //         let attachment = CreateAttachment::bytes(&*output_buf, filename);
    //         // let builder = CreateMessage::new().content("Test").add_file(attachment.expect("Attachment not provided"));
    //         let builder = CreateMessage::new()
    //             .content(format!("Reduced image size of {} from {} bytes to {} bytes", filename, img_size, attachment.data.len()))
    //             .add_file(attachment);
    //         let _ = msg.channel_id.send_message(&ctx.http, builder).await.expect("message did not send");
    //     }
        
    // }
    }

    // async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    //     if let Interaction::Command(command) = interaction {
    //         info!("Received command interaction: {:#?}", command);

    //         // let user_id: i64 = command.user.id.into();

    //         let response_content = match command.data.name.as_str() {
    //             "compress" => {
    //                 // let command = command.data.options.first().expect("Expected command");
    //                 // let command = command.data.options.first().expect("Expected command");
    //                 // match &command.value {

    //                 // }

    //                 let argument = command
    //                     .data
    //                     .options
    //                     .iter()
    //                     .find(|opt| opt.name == "image")
    //                     .cloned();

    //                 let value = argument.unwrap().value;
    //                 let image = value.as;

                    
    //             },
    //             command => unreachable!("Unknown command: {}", command),
    //         };
    //     }
    // }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // let guild_id = GuildId::new(self.guild_id.parse().unwrap());

        // let _ = guild_id
        //     .set_commands(
        //         &ctx.http,
        //         vec![
        //             CreateCommand::new("test").description("Just a test command for various purposes"),
        //             CreateCommand::new("compress")
        //             .description("Takes an image from the user and outputs a compressed image if under 8 MB")
        //             .add_option(
        //                 CreateCommandOption::new(
        //                     serenity::all::CommandOptionType::Attachment,
        //                     "image",
        //                     "Image to compress"
        //                 )
        //                 .required(true)
        //             ),
        //         ]
        //     )
        //     .await;
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

    // let guild_id = secrets
    //     .get("GUILD_ID")
    //     .context("'GUILD_ID' was not found")?;

    // let bot = Bot {
    //     guild_id
    // };

    // let client = Client::builder(&token, GatewayIntents::empty())
    //     .event_handler(bot)
    //     .await
    //     .expect("Err creating client");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
