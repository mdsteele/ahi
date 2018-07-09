extern crate ahi;
extern crate png;

use png::HasParameters;
use std::env;
use std::fs::File;
use std::io;
use std::path::Path;

fn convert_image(image: &ahi::Image, output_path: &Path) -> io::Result<()> {
    let rgba_data = image.rgba_data(ahi::Palette::default());
    let output_file = File::create(&output_path)?;
    let mut encoder =
        png::Encoder::new(output_file, image.width(), image.height());
    encoder.set(png::ColorType::RGBA).set(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgba_data).map_err(
        |err| match err {
            png::EncodingError::IoError(err) => err,
            png::EncodingError::Format(msg) => {
                io::Error::new(io::ErrorKind::InvalidData, msg.into_owned())
            }
        },
    )?;
    Ok(())
}

fn main() -> io::Result<()> {
    let num_args = env::args().count();
    if num_args != 2 {
        println!("Usage: ahi2png <path/to/file.ahi>");
        return Ok(());
    }

    let path_arg = env::args().nth(1).unwrap();
    let input_path = Path::new(&path_arg);
    let input_file = File::open(&input_path)?;
    let collection = ahi::Collection::read(input_file)?;
    if collection.images.len() == 1 {
        convert_image(&collection.images[0],
                      &input_path.with_extension("png"))?;
    } else {
        for (index, image) in collection.images.iter().enumerate() {
            let output_path =
                input_path.with_extension(format!("{}.png", index));
            convert_image(image, &output_path)?;
        }
    }
    Ok(())
}
