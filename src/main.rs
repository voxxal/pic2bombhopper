#![feature(int_abs_diff)]
#![feature(array_zip)]
use clap::Parser;
use image::GenericImageView;
use level::{Entity, Level, Point};
use log::{error, info, warn};
use raster::get_polygons;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    process::exit,
};

mod level;
mod raster;

enum Method {
    Raster, // Preforms bfs on the image, creating optomized polygons to use for levels
    Pixel,  // Every pixel is its own object
    Svg,    // Draws out svgs
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(value_hint = clap::ValueHint::FilePath)]
    image: PathBuf,
    /// Output path
    #[clap(short = 'o', value_parser, value_name = "path", value_hint = clap::ValueHint::FilePath, default_value = "./level.json")]
    output: PathBuf,
    /// Size of each pixel
    #[clap(short = 's', value_parser, value_name = "scale", default_value = "20")]
    scale: f32,

    /// Name of level
    #[clap(
        short = 'n',
        value_parser,
        value_name = "name",
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

    let mut method = Method::Pixel;

    match file_type {
        Some(kind) => match kind.mime_type() {
            "image/svg" => method = Method::Svg,
            _ => (),
        },
        None => {
            error!("File isn't an image.");
            exit(1)
        }
    }

    let image = match image::open(args.image) {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to load image ({}).", e);
            exit(1)
        }
    };
    match method {
        Method::Pixel => {
            info!(
                "Building level \"{}\" with pixel size of {}.",
                level.name, args.scale
            );
            // TODO don't forget to add the player
            for (x, y, color) in image.pixels() {
                let (x, y) = (x as f32, y as f32);
                let [red, green, blue, alpha] = color.0;
                if alpha == 0 {
                    continue;
                }
                level.push(Entity::Paint {
                    fill_color: red as i32 * 16_i32.pow(4)
                        + blue as i32 * 16_i32.pow(2)
                        + green as i32,
                    opacity: (alpha as f32) / 255.0,
                    vertices: vec![
                        Point::new(x * args.scale, y * args.scale),
                        Point::new((x + 1.0) * args.scale, y * args.scale),
                        Point::new((x + 1.0) * args.scale, (y + 1.0) * args.scale),
                        Point::new(x * args.scale, (y + 1.0) * args.scale),
                    ],
                })
            }

            if level.entities.len() >= 5000 {
                warn!("This method was designed for pixel art, consider using the Raster method for more optomized levels. This level has {} objects.", level.entities.len());
            }
        }
        Method::Raster => {
            let polygons = match get_polygons(image) {
                Ok(p) => p,
                Err(e) => {
                    error!("Something went wrong ({:?})", e);
                    exit(1)
                }
            };

            for (vertices, color) in polygons {
                let [red, green, blue, alpha] = color.0;
                if alpha == 0 {
                    continue;
                }
                level.push(Entity::Paint {
                    fill_color: red as i32 * 16_i32.pow(4)
                        + blue as i32 * 16_i32.pow(2)
                        + green as i32,
                    opacity: (alpha as f32) / 255.0,
                    vertices: vertices.into_iter().map(|p| p * args.scale).collect(),
                })
            }
        }
        Method::Svg => unimplemented!(),
    }

    let file = match File::create(args.output.clone()) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to create output file ({:?}).", e);
            exit(1)
        }
    };

    // TODO file optimization, rust spams 0.0 everywhere
    match serde_json::to_writer(BufWriter::new(file), &level) {
        Ok(_) => info!("Successfully wrote to {}.", args.output.to_str().unwrap()),
        Err(e) => {
            error!("Failed to write to file ({:?}).", e);
            exit(1)
        }
    }
}
