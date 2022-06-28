use std::collections::VecDeque;

use image::{DynamicImage, GenericImageView, Rgba};
use opencv::{
    core::{Scalar, Vec3b, VecN, Vector, CV_8UC1, CV_8UC3, no_array},
    imgproc::{
        cvt_color, draw_contours, find_contours_with_hierarchy, CHAIN_APPROX_SIMPLE,
        COLOR_BGR2GRAY, LINE_8, RETR_EXTERNAL,
    },
    prelude::*,
};

use crate::level::Point;

/// Does the funny raster thingy (https://github.com/kevinjycui/css-video)

fn approx(c1: Rgba<u8>, c2: Rgba<u8>, variance: u8) -> bool {
    c1.0.iter()
        .zip(c2.0.iter())
        .all(|(v1, v2)| v1.abs_diff(*v2) <= variance)
}

pub fn get_polygons(image: DynamicImage) -> Result<Vec<(Vec<Point>, Rgba<u8>)>, opencv::Error> {
    let (width, height) = image.dimensions();
    let mut visited = vec![vec![false; height as usize]; width as usize];
    let mut polygons: Vec<(Vec<Point>, Rgba<u8>)> = vec![];

    for (x, y, color) in image.pixels() {
        if visited[x as usize][y as usize] {
            continue;
        }
        let mut avg_color: Rgba<u32> = color.0.map(|v| v as u32).into();
        let mut img_seg = Mat::new_rows_cols_with_default(
            width as i32,
            height as i32,
            CV_8UC3,
            VecN::<f64, 4>([255.0, 255.0, 255.0, 255.0]),
        )?;

        let mut pixels = 1;
        let mut queue = VecDeque::from(vec![(x, y)]);
        img_seg.at_2d_mut::<Vec3b>(x as i32, y as i32)?.0 = [0, 0, 0];
        while !queue.is_empty() {
            let (x, y) = queue.pop_front().unwrap();
            let current_color = image.get_pixel(x, y);
            avg_color.0 = avg_color.0.zip(current_color.0).map(|(a, b)| a + b as u32);
            pixels += 1;

            if x as i32 - 1 >= 0 {
                img_seg.at_2d_mut::<Vec3b>(x as i32 - 1, y as i32)?.0 = [0, 0, 0];
                if !visited[x as usize - 1][y as usize]
                    && approx(current_color, image.get_pixel(x - 1, y), 25)
                {
                    visited[x as usize - 1][y as usize] = true;
                    queue.push_back((x - 1, y));
                }
            }

            if x + 1 < width {
                img_seg.at_2d_mut::<Vec3b>(x as i32 + 1, y as i32)?.0 = [0, 0, 0];
                if !visited[x as usize + 1][y as usize]
                    && approx(current_color, image.get_pixel(x + 1, y), 25)
                {
                    visited[x as usize + 1][y as usize] = true;
                    queue.push_back((x + 1, y));
                }
            }

            if y as i32 - 1 >= 0 {
                img_seg.at_2d_mut::<Vec3b>(x as i32, y as i32 - 1)?.0 = [0, 0, 0];
                if !visited[x as usize][y as usize - 1]
                    && approx(current_color, image.get_pixel(x, y - 1), 25)
                {
                    visited[x as usize][y as usize - 1] = true;
                    queue.push_back((x, y - 1));
                }
            }

            if y + 1 < height {
                img_seg.at_2d_mut::<Vec3b>(x as i32, y as i32 + 1)?.0 = [0, 0, 0];
                if !visited[x as usize][y as usize + 1]
                    && approx(current_color, image.get_pixel(x, y + 1), 25)
                {
                    visited[x as usize][y as usize + 1] = true;
                    queue.push_back((x, y + 1));
                }
            }
        }

        if true {
            let final_color = Rgba::from(avg_color.0.map(|v| (v / pixels as u32) as u8));
            let mut contours: Vector<Vector<opencv::core::Point>> = Vector::default();
            let mut _hierarchy = Mat::default();
            let mut final_seg = Mat::new_rows_cols_with_default(
                width as i32,
                height as i32,
                CV_8UC1,
                VecN::<f64, 4>([255.0, 255.0, 255.0, 255.0]),
            )?;
            cvt_color(&img_seg, &mut final_seg, COLOR_BGR2GRAY, 0)?;
            find_contours_with_hierarchy(
                &final_seg,
                &mut contours,
                &mut _hierarchy,
                RETR_EXTERNAL,
                CHAIN_APPROX_SIMPLE,
                opencv::core::Point::default(),
            )?;

            opencv::highgui::imshow("Contours", &final_seg)?;
            opencv::imgproc::draw_contours(
                &mut final_seg,
                &contours,
                -1,
                Scalar::default(),
                3,
                LINE_8,
                &no_array(),
                i32::MAX,
                opencv::core::Point::default(),
            )?;
            opencv::highgui::wait_key(0)?;

            for contour in contours.iter() {
                let mut points = vec![];
                for point in contour {
                    points.push(Point::new(point.x as f32, point.y as f32));
                }

                polygons.push((points, final_color));
            }
        }
    }

    Ok(polygons)
}
