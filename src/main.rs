use clap::Parser;
use image::GenericImageView;
use level::{Entity, Level, Params, Point};
use log::{error, info, warn};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    process::exit,
};

mod level;

enum Methods {
    Pixel,
    Svg, // Draws out svgs
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(value_hint = clap::ValueHint::FilePath)]
    image: PathBuf,
    /// Output path
    #[clap(short = 'o', value_parser, value_name = "PATH", value_hint = clap::ValueHint::FilePath, default_value = "./level.json")]
    output: PathBuf,
    /// Size of each pixel
    #[clap(
        short = 'g',
        value_parser,
        value_name = "GRID-SIZE",
        default_value = "20"
    )]
    pixel_size: f32,

    /// Name of level
    #[clap(
        short = 'n',
        value_parser,
        value_name = "NAME",
        default_value = "Created with pic2bombhopper"
    )]
    name: String,
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .format_module_path(false)
        .format(|buf, record| {
            writeln!(
                buf,
                "[ {} ] {}",
                buf.default_styled_level(record.level()),
                record.args()
            )
        })
        .init();
    let args = Args::parse();
    let mut level = Level::new(args.name, [0, 0]);
    let file_type = match infer::get_from_path(args.image.clone()) {
        Ok(t) => t,
        Err(e) => {
            error!("File Doesn't exist ({}).", e);
            exit(1)
        }
    };
    let mut method = Methods::Pixel;

    match file_type {
        Some(kind) => match kind.mime_type() {
            "image/svg" => method = Methods::Svg,
            _ => (),
        },
        None => {
            error!("File isn't an image.");
            exit(1)
        }
    }

    match method {
        Methods::Pixel => {
            let image = match image::open(args.image) {
                Ok(i) => i,
                Err(e) => {
                    error!("Failed to load image ({}).", e);
                    exit(1)
                }
            };

            info!(
                "Building level \"{}\" with pixel size of {}.",
                level.name, args.pixel_size
            );

            for (x, y, pixel) in image.pixels() {
                let (x, y) = (x as f32, y as f32);
                let [red, green, blue, alpha] = pixel.0;
                if alpha == 0 {
                    continue;
                }
                level.push(Entity {
                    r#type: String::from("paint"),
                    params: Params::Paint {
                        fill_color: red as i32 * 16_i32.pow(4)
                            + blue as i32 * 16_i32.pow(2)
                            + green as i32,
                        opacity: (alpha as f32) / 255.0,
                        vertices: vec![
                            Point::new(x * args.pixel_size, y * args.pixel_size),
                            Point::new((x + 1.0) * args.pixel_size, y * args.pixel_size),
                            Point::new((x + 1.0) * args.pixel_size, (y + 1.0) * args.pixel_size),
                            Point::new(x * args.pixel_size, (y + 1.0) * args.pixel_size),
                        ],
                    },
                })
            }

            if level.entities.len() >= 5000 {
                warn!("This tool was created for pixel art. Using bigger images may lag the game and create large files. This level has {} objects.", level.entities.len())
            }

            let file = match File::create(args.output.clone()) {
                Ok(f) => f,
                Err(e) => {
                    error!("Failed to create output file ({:?}).", e);
                    exit(1)
                }
            };

            // TODO file optimization, rust spams 0.0 everywhere
            let writer: BufWriter<File> = BufWriter::new(file);
            match serde_json::to_writer(writer, &level) {
                Ok(_) => info!("Successfully wrote to {}.", args.output.to_str().unwrap()),
                Err(e) => {
                    error!("Failed to write to file ({:?}).", e);
                    exit(1)
                }
            }
        }
        Methods::Svg => unimplemented!(),
    }
}
