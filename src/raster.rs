use std::collections::VecDeque;

use image::{DynamicImage, GenericImageView, GrayImage, ImageBuffer, Luma, Rgba};
use imageproc::contours::{find_contours, Contour};

use bombhopper::Point;

/// Does the funny raster thingy (https://github.com/kevinjycui/css-video)

fn approx(c1: Rgba<u8>, c2: Rgba<u8>, variance: u8) -> bool {
    c1.0.iter()
        .zip(c2.0.iter())
        .all(|(v1, v2)| v1.abs_diff(*v2) <= variance)
}

pub fn get_polygons(
    image: DynamicImage,
    variance: u8,
    lower_cut: i32,
) -> Vec<(Vec<Point>, Rgba<u8>)> {
    let (width, height) = image.dimensions();
    let mut visited = vec![vec![false; width as usize]; height as usize];
    let mut polygons: Vec<(Vec<Point>, Rgba<u8>)> = vec![];

    for (x, y, color) in image.pixels() {
        if visited[x as usize][y as usize] {
            continue;
        }
        let mut avg_color: Rgba<u32> = color.0.map(|v| v as u32).into();
        let mut segment: GrayImage = ImageBuffer::from_pixel(width + 1, height + 1, Luma([0]));

        let mut pixels = 0;
        let mut queue = VecDeque::from(vec![(x, y)]);
        segment.get_pixel_mut(x, y).0 = [255];

        while !queue.is_empty() {
            let (x, y) = queue.pop_front().unwrap();
            let current_color = image.get_pixel(x, y);
            avg_color.0 = avg_color.0.zip(current_color.0).map(|(a, b)| a + b as u32);
            pixels += 1;

            if x as i32 - 1 >= 0 {
                segment.get_pixel_mut(x - 1, y).0 = [255];
                if !visited[x as usize - 1][y as usize]
                    && approx(current_color, image.get_pixel(x - 1, y), variance)
                {
                    visited[x as usize - 1][y as usize] = true;
                    queue.push_back((x - 1, y));
                }
            }

            if x + 1 < width  {
                segment.get_pixel_mut(x + 1, y).0 = [255];
                if !visited[x as usize + 1][y as usize]
                    && approx(current_color, image.get_pixel(x + 1, y), variance)
                {
                    visited[x as usize + 1][y as usize] = true;
                    queue.push_back((x + 1, y));
                }
            }

            if y as i32 - 1 >= 0 {
                segment.get_pixel_mut(x, y - 1).0 = [255];
                if !visited[x as usize][y as usize - 1]
                    && approx(current_color, image.get_pixel(x, y - 1), variance)
                {
                    visited[x as usize][y as usize - 1] = true;
                    queue.push_back((x, y - 1));
                }
            }

            if y + 1 < height {
                segment.get_pixel_mut(x, y + 1).0 = [255];
                if !visited[x as usize][y as usize + 1]
                    && approx(current_color, image.get_pixel(x, y + 1), variance)
                {
                    visited[x as usize][y as usize + 1] = true;
                    queue.push_back((x, y + 1));
                }
            }
        }

        assert!(avg_color
            .0
            .iter()
            .all(|v| (*v / (pixels as u32 + 1)) <= 255));

        // Ignore orphen pixels
        if pixels > lower_cut {
            let final_color = Rgba::from(avg_color.0.map(|v| (v / (pixels as u32 + 1)) as u8));
            let contours: Vec<Contour<i32>> = find_contours(&segment);

            // segment.save(format!("segments/segment_{:?}.png", final_color));
            for contour in contours.into_iter() {
                polygons.push((
                    contour
                        .points
                        .into_iter()
                        .map(|p| Point::new(p.x as f32, p.y as f32))
                        .collect(),
                    final_color,
                ));
            }
        }
    }
    polygons
}
