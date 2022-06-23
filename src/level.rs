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

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Bombs {
    Empty,
    Bomb,
    Grenade,
}

#[derive(Serialize)]
#[serde(untagged)] 
pub enum Params {
    Player {
        isStatic: bool,
        angle: i32,
        x: f32,
        y: f32,
        magazine: Vec<Bombs>,
    },
    Paint {
        fill_color: i32,
        opacity: f32,
        vertices: Vec<Point>,
    },
}

#[derive(Serialize)]
pub struct Entity {
    pub r#type: String,
    pub params: Params,
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
