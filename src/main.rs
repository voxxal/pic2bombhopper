#![feature(array_zip)]
use bombhopper::{Entity, Level};
use clap::Parser;
use log::{error, info};
use pic2bombhopper::{optimize, raster::get_polygons};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    process::exit,
};

pub const NEIGHBORS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

enum Method {
    Raster, // Preforms bfs on the image, creating optomized polygons to use for levels
    Svg,    // Draws out svgs
}

#[derive(Debug, Parser)]
struct Args {
    /// Name of level
    #[clap(
        short = 'n',
        value_parser,
        value_name = "name",
        default_value = "Created with pic2bombhopper"
    )]
    name: String,

    #[clap(value_hint = clap::ValueHint::FilePath)]
    image: PathBuf,
    /// Output path
    #[clap(short = 'o', value_parser, value_name = "path", value_hint = clap::ValueHint::FilePath, default_value = "./level.json")]
    output: PathBuf,
    /// Size of each pixel
    #[clap(short = 's', value_parser, value_name = "scale", default_value = "10")]
    scale: f32,
    // How close the color of 2 pixels need to be for them to be merged
    #[clap(
        short = 'v',
        value_parser,
        value_name = "variance",
        default_value = "6"
    )]
    variance: u8,
    // If a polygon consists of less than this many pixels, remove it
    #[clap(
        short = 'c',
        value_parser,
        value_name = "lower-cut",
        default_value = "8"
    )]
    lower_cut: i32,
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

    let mut method = Method::Raster;

    match file_type {
        Some(kind) => match kind.mime_type() {
            "image/svg" => method = Method::Svg,
            _ => (),
        },
        None => {
            error!("File doesn't have file type.");
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
        Method::Raster => {
            for (vertices, color) in get_polygons(image, args.variance, args.lower_cut) {
                let [red, green, blue, alpha] = color.0;
                if alpha == 0 {
                    continue;
                }

                level.push(Entity::Paint {
                    fill_color: red as i32 * 16_i32.pow(4)
                        + green as i32 * 16_i32.pow(2)
                        + blue as i32,
                    opacity: (alpha as f32) / 255.0,
                    vertices: vertices.into_iter().map(|p| p * args.scale).collect(),
                })
            }
        }
        Method::Svg => unimplemented!(),
    }

    optimize::prune_aligned_vertices(&mut level);

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
