#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use bombhopper::{Entity, Level};
use eframe::{
    egui::{self, RichText, Ui},
    emath::Numeric,
    epaint::Color32,
};
use image::{DynamicImage, GenericImageView};
use pic2bombhopper::{optimize, raster::get_polygons};
use rfd::FileDialog;
use std::{
    fs::File,
    io::{BufWriter, Write},
    ops::RangeInclusive,
    path::PathBuf,
};

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

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "pic2bombhopper",
        options,
        Box::new(|_cc| Box::new(Pic2Bombhopper::default())),
    );
}

struct Pic2Bombhopper {
    name: String,
    image_path: Option<PathBuf>,
    scale: f32,
    variance: u8,
    lower_cut: i32,

    status: Option<Result<String, String>>,
}

impl Default for Pic2Bombhopper {
    fn default() -> Self {
        Self {
            name: "Created with pic2bombhopper".to_owned(),
            image_path: None,
            scale: 10.0,
            variance: 6,
            lower_cut: 8,

            status: None,
        }
    }
}

impl eframe::App for Pic2Bombhopper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("pic2bombhopper");
            ui.horizontal(|ui| {
                ui.label("Level name: ");
                ui.text_edit_singleline(&mut self.name);
            });

            ui.horizontal(|ui| {
                ui.label("Select image: ");
                if ui.button("Open file...").clicked() {
                    if let Some(path) = FileDialog::new().pick_file() {
                        self.image_path = Some(path.clone());
                    }
                }
            });

            if let Some(picked_path) = &self.image_path {
                ui.horizontal(|ui| {
                    ui.label("Picked file:");
                    ui.monospace(picked_path.display().to_string());
                });
            }

            ui.strong("Options:");
            // TODO move into function (rule of 3s)
            slider_option(
                ui,
                &mut self.scale,
                "Scale: ",
                "Also known as grid size",
                1.0..=40.0,
            );

            slider_option(
                ui,
                &mut self.variance,
                "Variance: ",
                "How close the color of 2 pixels need to be for them to be merged",
                0..=48,
            );

            slider_option(
                ui,
                &mut self.lower_cut,
                "Lower cut: ",
                "If a polygon consists of less than this many pixels, remove it",
                0..=32,
            );

            ui.label("\n");

            // let button_name = if let Some(img) = &self.image {
            //     let (x, y) = img.dimensions();
            //     let total_pixels = x.saturating_mul(y);
            //     if total_pixels > 5_000_000 {
            //         "Nuke my computer baby!"
            //     } else if total_pixels > 1_000_000 {
            //         "Generate pain and suffering (for your computer)"
            //     } else {
            //         "Generate Map"
            //     }
            // } else {
            //     "Generate Map"
            // };

            // i don't know how to improve this but i will :) probably involves moving stuff into functions
            if ui.button("Generate Map").clicked() {
                if let Some(path) = &self.image_path {
                    match image::open(path) {
                        Ok(image) => {
                            let mut level = Level::new(self.name.clone(), [0, 0]);

                            for (vertices, color) in
                                get_polygons(image, self.variance, self.lower_cut)
                            {
                                let [red, green, blue, alpha] = color.0;
                                if alpha == 0 {
                                    continue;
                                }

                                level.push(Entity::Paint {
                                    fill_color: red as i32 * 16_i32.pow(4)
                                        + green as i32 * 16_i32.pow(2)
                                        + blue as i32,
                                    opacity: (alpha as f32) / 255.0,
                                    vertices: vertices
                                        .into_iter()
                                        .map(|p| p * self.scale)
                                        .collect(),
                                })
                            }

                            optimize::prune_aligned_vertices(&mut level);

                            if let Some(path) = rfd::FileDialog::new()
                                .set_file_name("level.json")
                                .save_file()
                            {
                                let file = match File::create(path.clone()) {
                                    Ok(f) => Some(f),
                                    Err(e) => {
                                        self.status = Some(Err(format!(
                                            "Failed to create output file ({:?}).",
                                            e
                                        )));
                                        None
                                    }
                                };

                                if let Some(f) = file {
                                    match serde_json::to_writer(BufWriter::new(f), &level) {
                                        Ok(_) => {
                                            self.status = Some(Ok(format!(
                                                "Successfully wrote to {}.",
                                                path.display().to_string()
                                            )))
                                        }
                                        Err(e) => {
                                            self.status = Some(Err(format!(
                                                "Failed to write to file ({:?}).",
                                                e
                                            )));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            self.status = Some(Err(format!("Failed to open image file ({:?})", e)));
                        }
                    }
                } else {
                    self.status = Some(Err(String::from(
                        "You need to select an image before running this program",
                    )))
                }
            }

            if let Some(status) = &self.status {
                match status {
                    Ok(s) => {
                        ui.label(RichText::new(s).color(Color32::LIGHT_GREEN));
                    }
                    Err(s) => {
                        ui.label(RichText::new(s).color(Color32::LIGHT_RED));
                    }
                }
            }
        });
    }
}

fn slider_option<Num: Numeric>(
    ui: &mut Ui,
    variable: &mut Num,
    name: &str,
    description: &str,
    range: RangeInclusive<Num>,
) {
    ui.horizontal(|ui| {
        ui.label(name);
        ui.add(egui::Slider::new(variable, range));
        ui.label(RichText::new(description).italics().color(Color32::GRAY));
    });
}
