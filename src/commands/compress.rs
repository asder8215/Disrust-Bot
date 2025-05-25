use crate::structs::message::Message;

use std::io::Cursor;
use image::{DynamicImage, ImageReader};
use mozjpeg::{Compress, ColorSpace};
use serenity::all::{ResolvedOption, ResolvedValue};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::{all::CreateAttachment};

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
    match load_image_from_bytes(pixels.clone()) {
        Ok(loaded_img) => {
            // loaded_img
            let rgb_data = convert_to_rgb(loaded_img);
            let mut comp = Compress::new(ColorSpace::JCS_RGB);
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
    }
}

pub async fn run(options: &[ResolvedOption<'_>]) -> Message {
    if let Some(ResolvedOption {
        value: ResolvedValue::Attachment(attachment), ..
    }) = options.first()
    {
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
                        
                        if compressed_data.len() < 8388608 {
                            Message{
                                text: format!(
                                    "Reduced image size of {} from {} bytes to {} bytes",
                                    filename,
                                    size,
                                    compressed_data.len()
                                ),
                                attachment: Some(compressed_attachment)
                            }   
                        } else {
                            Message{
                                text: "Could not compress image to <= 8 MB as per Discord's upload file limit".to_string(),
                                attachment: Some(compressed_attachment)
                            }
                        }
                    }
                    Err(e) => {
                        Message{
                            text: format!("Failed to compress image: {}", e),
                            attachment: None
                        }
                    }
                }
        } else {
            Message{
                text: "Image dimensions not found.".to_string(),
                attachment: None
            }
        }
    } else {
        Message{
            text: "Please provide a valid attachment".to_string(),
            attachment: None
        }
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("compress")
        .description("Takes an image from the user and outputs a compressed image if under 8 MB")
        .add_option(
            CreateCommandOption::new(
                serenity::all::CommandOptionType::Attachment,
                "image",
                "Image to compress"
            )
            .required(true)
        )
}