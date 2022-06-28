use std::ops::{Add, Sub, Mul};

use serde::Serialize;

#[derive(Serialize)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Self;

    fn mul(self, other: f32) -> Self::Output {
        Point {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Bombs {
    Empty,
    Bomb,
    Grenade,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "params")] 
pub enum Entity {
    #[serde(rename_all = "camelCase")]
    Player {
        is_static: bool,
        angle: i32,
        x: f32,
        y: f32,
        magazine: Vec<Bombs>,
    },
    #[serde(rename_all = "camelCase")]
    Paint {
        fill_color: i32,
        opacity: f32,
        vertices: Vec<Point>,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub name: String,
    pub timings: [i32; 2],
    pub entities: Vec<Entity>,
    format_version: u8,
}

impl Level {
    pub fn new(name: String, timings: [i32; 2]) -> Self {
        Self {
            name,
            timings,
            entities: vec![],
            format_version: 0,
        }
    }

    pub fn push(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}
