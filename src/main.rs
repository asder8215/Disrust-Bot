use std::io::{Read, Write};
use std::process::{Command, Stdio};

use anyhow::Context as _;
use serenity::all::{CreateCommand, CreateCommandOption, CreateMessage, Interaction};
use serenity::{all::CreateAttachment, async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};
use mozjpeg;

// struct Bot{
//     guild_id: String
// }


struct Bot;

struct MyImage {
    width: usize,
    height: usize,
    pixels: Vec<u8>
}

impl MyImage {
    /// Optimize image to jpeg, the quality 60-80 are recommended.
    pub async fn to_mozjpeg(&self, quality: u8) -> Result<Vec<u8>, Box<dyn std::error::Error + Send>> {
        let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
        comp.set_size(self.width, self.height);
        comp.set_quality(quality as f32);
        let mut comp = comp.start_compress(Vec::new()).expect("Data did not start compressing");
        comp.write_scanlines(&*self.pixels).expect("Scanlines were not written");
        let data = comp.finish().expect("Data did not finish compressing");
        Ok(data)
    }
}

#[async_trait]
impl EventHandler for Bot {

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!").await {
                error!("Error sending message: {:?}", e);
            }
        }
        // else if msg.content.starts_with("!test") {
        //     if let Some(image) = msg.attachments.get(0) {
        //         // Ensure the attachment is an image
        //         if let Some(img_width) = image.width {
        //             if let Some(img_height) = image.height {
        //                 // Download image data
        //                 let response = reqwest::get(&image.url).await.expect("Failed to download image");
        //                 let img_bytes = response.bytes().await.expect("Failed to read image bytes");
                        
        //                 // Prepare image for compression
        //                 let img_to_compress = turbojpeg::Image {
        //                     pixels: &img_bytes.to_vec()[..],
        //                     width: img_width as usize,
        //                     pitch: img_width as usize * turbojpeg::PixelFormat::RGB.size(),
        //                     height: img_height as usize,
        //                     format: turbojpeg::PixelFormat::RGB,
        //                 };
        
        //                 // Compress image
        //                 let mut compressor = turbojpeg::Compressor::new().expect("Failed to create compressor");
        //                 compressor.set_quality(70).expect("Failed to set quality");
        //                 compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2).expect("Failed to set subsampling");
        
        //                 // let mut output_buf = turbojpeg::OutputBuf::new_owned();
        //                 let output_buf = compressor.compress_to_vec(img_to_compress).expect("Failed to compress image");
        
        //                 // Create attachment for compressed image
        //                 let attachment = CreateAttachment::bytes(&*output_buf, &image.filename);
        
        //                 // Send response message
        //                 let builder = CreateMessage::new()
        //                     .content(format!(
        //                         "Reduced image size of {} from {} bytes to {} bytes",
        //                         &image.filename,
        //                         image.size,
        //                         output_buf.len()
        //                     ))
        //                     .add_file(attachment);
        //                 let _ = msg.channel_id.send_message(&ctx.http, builder).await.expect("Failed to send message");
        //             } else {
        //                 let _ = msg.channel_id.say(&ctx.http, "Image dimensions not found.").await;
        //             }
        //         } else {
        //             let _ = msg.channel_id.say(&ctx.http, "Attachment is not an image.").await;
        //         }
        //     } else {
        //         let _ = msg.channel_id.say(&ctx.http, "No attachments found.").await;
        //     }
        // }
        else if msg.content.starts_with("!test") {
            if let Some(image) = msg.attachments.get(0) {
                // Ensure the attachment is an image
                if let (Some(img_width), Some(img_height)) = (image.width, image.height) {
                        // Download image data
                        let response = reqwest::get(&image.url).await.expect("Failed to download image");
                        let img_bytes = response.bytes().await.expect("Failed to read image bytes");
    
                        // Compress the image using mozjpeg's cjpeg tool, directly in memory
                        // let status = Command::new("cjpeg")
                        //     .arg("-quality")
                        //     .arg("70") // Set the quality
                        //     .arg("-outfile") // Output to stdout
                        //     .arg("-")
                        //     .stdin(Stdio::piped()) // Pipe input
                        //     .stdout(Stdio::piped()) // Pipe output
                        //     .spawn()
                        //     .expect("Failed to spawn cjpeg process");

                        // let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
                        // comp.set_size(img_width as usize, img_height as usize);
                        // comp.set_quality(70.0);
                        // let mut comp = comp.start_compress(Vec::new()).expect("Compress did not start");
                        // // let mut output_buf = Vec::new();

                        // comp.write_scanlines(&img_bytes.to_vec()[..]).expect("Did not write scanlines");

                        // let data = comp.finish().expect("Did not finish compressing data");

                        let input_image = MyImage {
                            width: img_width as usize,
                            height: img_height as usize,
                            pixels: img_bytes.to_vec(), // Raw RGB pixel data here
                        };

                        match input_image.to_mozjpeg(70).await {
                            Ok(compressed_data) => {
                                let attachment = CreateAttachment::bytes(&*compressed_data, &image.filename);

                                // Send the compressed image
                                let builder = CreateMessage::new()
                                    .content(format!(
                                        "Reduced image size of {} from {} bytes to {} bytes",
                                        &image.filename,
                                        image.size,
                                        compressed_data.len()
                                    ))
                                    .add_file(attachment);
            
                                let _ = msg.channel_id
                                    .send_message(&ctx.http, builder)
                                    .await
                                    .expect("Failed to send message");
                            }
                            Err(e) => {
                                let _ = msg
                                    .channel_id
                                    .say(&ctx.http, format!("Failed to compress image: {}", e))
                                    .await;
                            }
                        }
                        // let mut child = status; // This is the process handle
    
                        // // Write the image bytes to the stdin of cjpeg
                        // {
                        //     let mut stdin = child.stdin.as_mut().expect("Failed to open stdin");
                        //     stdin.write_all(&img_bytes).expect("Failed to write to stdin");
                        // }
    
                        // // Read the compressed image from the stdout of cjpeg
                        // let output_bytes = {
                        //     let mut stdout = child.stdout.take().expect("Failed to open stdout");
                        //     let mut output = Vec::new();
                        //     stdout.read_to_end(&mut output).expect("Failed to read from stdout");
                        //     output
                        // };
    
                        // // Wait for the cjpeg process to finish
                        // let _ = child.wait().expect("cjpeg process failed");
    
                        // Create the attachment for the compressed image
                        // let attachment = CreateAttachment::bytes(&*data, &image.filename);
    
                        // // Send the compressed image
                        // let builder = CreateMessage::new()
                        //     .content(format!(
                        //         "Reduced image size of {} from {} bytes to {} bytes",
                        //         &image.filename,
                        //         image.size,
                        //         1024
                        //     ))
                        //     .add_file(attachment);
    
                        // let _ = msg.channel_id.send_message(&ctx.http, builder).await.expect("Failed to send message");
                } else {
                    let _ = msg.channel_id.say(&ctx.http, "Image dimensions not found.").await;
                }
            }
            else {
                let _ = msg.channel_id.say(&ctx.http, "Attachment is not an image.").await;
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
