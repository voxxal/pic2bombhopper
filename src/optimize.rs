use bombhopper::{Entity, Level, Point};

fn points_aligned(point1: &Point, point2: &Point, point3: &Point) -> bool {
    (point3.y - point1.y) * (point2.x - point1.x) == (point2.y - point1.y) * (point3.x - point1.x)
}

// https://github.com/getkey/ble/blob/339cd2028346f1198a476734c7f12d10912177c1/src/models/VerticesParams.ts#L107
pub fn prune_aligned_vertices(level: &mut Level) {
    for entity in &mut level.entities {

        if let Entity::Paint { ref mut vertices, .. } = entity {
            let mut i = 0;
            while vertices.len() > 2 && i < vertices.len() {
                let current = vertices[i];
                let next = vertices[(i + 1) % vertices.len()];
                let later = vertices[(i + 2) % vertices.len()];
                if points_aligned(&current, &next, &later) {
                    vertices.remove(i + 1);
                } else {
                    i += 1;
                }
            }
        }
    }
}
