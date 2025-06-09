use crate::structs::message::Message;

use image::{ImageFormat, ImageReader};
use oxipng::{Options, optimize_from_memory};
use serenity::all::CreateAttachment;
use serenity::all::{ResolvedOption, ResolvedValue};
use serenity::builder::{CreateCommand, CreateCommandOption};
use std::io::Cursor;
use webp::Encoder as WebPEncoder;

const MAX_IMG_SIZE: usize = 8388608;

/// Load image from Vec<u8> into a DynamicImage
fn load_image_from_bytes(
    pixels: &[u8],
) -> Result<ImageReader<Cursor<&[u8]>>, Box<dyn std::error::Error>> {
    Ok(ImageReader::new(Cursor::new(pixels))
        .with_guessed_format()
        .map_err(|error| format!("Could not guess image format: {error}"))?)
}

/// Performs compression/optimization on the image for png, jpg, webp, and gif format
async fn compressed_image(
    pixels: &[u8],
    quality: u8,
    preset_level: u8,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // propagates the error from load_image_from_bytes if occurs
    // otherwise it's OK!
    let reader = load_image_from_bytes(pixels)?;

    let format_type = reader.format().ok_or("Could not determine image format")?;

    let image = reader
        .decode()
        .map_err(|error| format!("Could not decode image: {error}"))?;

    match format_type {
        ImageFormat::Png => {
            // use default optimization if no preset val given
            if preset_level == 0 {
                return Ok(optimize_from_memory(pixels, &Options::default())?);
            }
            Ok(optimize_from_memory(
                pixels,
                &Options::from_preset(preset_level),
            )?)
        }
        ImageFormat::Jpeg => {
            // Sourced from: https://github.com/vicanso/imageoptimize/blob/670bd3f40851c87d5fea5bb3e0e841d98a404c60/src/images.rs#L285
            let rgb_data = image.as_bytes(); // use a slice rather than vec bc waste of making a copy
            let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
            comp.set_size(image.width() as usize, image.height() as usize);
            comp.set_quality(quality as f32);
            let mut comp = comp
                .start_compress(Vec::new())
                .map_err(|error| format!("Data did not start compressing: {error}"))?;
            comp.write_scanlines(rgb_data)
                .map_err(|error| format!("Scanlines were not written: {error}"))?;
            let data = comp
                .finish()
                .map_err(|error| format!("Data did not finish compressing: {error}"))?;
            Ok(data)
        }
        ImageFormat::WebP => {
            // sourced from here: https://github.com/jaredforth/webp/blob/main/examples/convert.rs
            let encoder = WebPEncoder::from_image(&image)?;
            let webp = encoder.encode(quality.into());
            Ok(Vec::from(&*webp))
        }
        ImageFormat::Gif => {
            todo!()
        }
        _ => Err("Unsupported image type".into()),
    }
}

pub async fn run(options: &[ResolvedOption<'_>]) -> Message {
    if let Some(ResolvedOption {
        value: ResolvedValue::Attachment(attachment),
        ..
    }) = options.first()
    {
        // Ensure the attachment is an image
        // Download image data
        let filename = &attachment.filename;
        let size = attachment.size as usize;
        let img_bytes = attachment
            .download()
            .await
            .map_err(|error| format!("Could not download image: {error}"));

        let img_bytes = match img_bytes {
            Ok(img) => img,
            Err(e) => {
                return Message {
                    text: e,
                    attachment: None,
                };
            }
        };

        let quality = match options.get(1) {
            Some(ResolvedOption {
                value: ResolvedValue::Integer(integer),
                ..
            }) => *integer as u8,
            Some(_) => {
                return Message {
                    text: "Invalid quality level".to_string(),
                    attachment: None,
                };
            }
            None => 70,
        };

        let preset_lvl = match options.get(2) {
            Some(ResolvedOption {
                value: ResolvedValue::Integer(integer),
                ..
            }) => *integer as u8,
            Some(_) => {
                return Message {
                    text: "Invalid preset level".to_string(),
                    attachment: None,
                };
            }
            None => 0,
        };

        // Attempt compressing image
        match compressed_image(&img_bytes, quality, preset_lvl).await {
            Ok(compressed_data) => {
                // Testing purposes to see if I can see any difference between original image size and compressed image
                // size

                let compressed_data_size = compressed_data.len();
                if compressed_data_size < MAX_IMG_SIZE && compressed_data_size < size {
                    let compressed_attachment =
                        CreateAttachment::bytes(compressed_data.clone(), filename);
                    Message {
                        text: format!(
                            "Reduced image size of {} from {} bytes to {} bytes",
                            filename,
                            size,
                            compressed_data.len()
                        ),
                        attachment: Some(compressed_attachment),
                    }
                } else if compressed_data_size >= size {
                    Message {
                        text: "Could not compress image any further".to_string(),
                        attachment: None,
                    }
                } else {
                    Message {
                        text: "Could not compress image to less than 8 MB".to_string(),
                        attachment: None,
                    }
                }
            }
            Err(e) => Message {
                text: format!("Failed to compress image: {}", e),
                attachment: None,
            },
        }
    } else {
        Message {
            text: "Please provide a valid attachment".to_string(),
            attachment: None,
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
                "Image to compress",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                serenity::all::CommandOptionType::Integer,
                "quality",
                "Quality level for jpg and webp to suppress to",
            )
            .required(false)
            .min_int_value(1)
            .max_int_value(100),
        )
        .add_option(
            CreateCommandOption::new(
                serenity::all::CommandOptionType::Integer,
                "preset",
                "Png preset level for png (1-6); default compression used otherwise",
            )
            .required(false)
            .min_int_value(1)
            .max_int_value(6),
        )
}
