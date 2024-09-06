use crate::composition::material::Material;
use crate::composition::objects::object::Object;
use crate::rendering::hit::Hit;
use crate::rendering::ray::Ray;
use crate::vector::Vector;
use serde::Deserialize;

const E: f64 = 0.0000001;

#[derive(Deserialize)]
pub struct Cube {
    pub c: Vector<f64, 3>, // Centre
    pub d: f64,  // Distance from centre to face centres
    pub material: Material,
}

enum CubeFace {
    Front,
    Back,
    Bottom,
    Top,
    Left,
    Right,
}

impl CubeFace {
    fn orientation(&self) -> f64 {
        match *self {
            CubeFace::Front => { -1.0 }
            CubeFace::Back => { 1.0 }
            CubeFace::Bottom => { -1.0 }
            CubeFace::Top => { 1.0 }
            CubeFace::Left => { -1.0 }
            CubeFace::Right => { 1.0 }
        }
    }

    fn plane_coordinate(&self) -> usize {
        match *self {
            CubeFace::Front => { 2 }
            CubeFace::Back => { 2 }
            CubeFace::Bottom => { 1 }
            CubeFace::Top => { 1 }
            CubeFace::Left => { 0 }
            CubeFace::Right => { 0 }
        }
    }
}


impl Cube {
    fn in_face(&self, orientation: f64, i: usize, ray: &Ray) -> Option<Hit> {
        if ray.d[i].abs() < E {
            return None;
        }
        let t = (self.c[i] - ray.s[i] + orientation * self.d) / ray.d[i];
        let h = ray.at(t);

        let i1 = (i + 1) % 3;
        let i2 = (i + 2) % 3;
        let in_1 = h[i1] >= self.c[i1] - (self.d + E) && h[i1] <= self.c[i1] + (self.d + E);
        let in_2 = h[i2] >= self.c[i2] - (self.d + E) && h[i2] <= self.c[i2] + (self.d + E);

        // Create normal vector from face information.
        let mut n = [0.0, 0.0, 0.0];
        n[i] = orientation;
        let mut material: [u8; 3] = [0, 0, 0];
        for i in 0..3 {
            material[i] = (n[i].abs() * 255.0) as u8;
        }

        if in_1 && in_2 {
            return Some(Hit {
                loc: h,
                t,
                normal: Vector::new(n),
                material: self.material.clone(),
            });
        }
        return None;
    }

    fn intersect_face(&self, ray: &Ray) -> Option<Hit> {
        let faces = vec![
            CubeFace::Front, // Front (-1, -1, -1) to (1, 1, -1)
            CubeFace::Back, // Back  (-1, -1, 1) to (1, 1, 1)
            CubeFace::Left, // Left (-1, -1, -1) to (-1, 1, 1)
            CubeFace::Right, // Right (1, -1, -1) to (1, 1, 1)
            CubeFace::Bottom, // Bottom (-1, -1, -1) to (1, -1, 1)
            CubeFace::Top, // Top (-1, 1, -1) to (1, 1, 1)
        ];

        let mut closest: Option<Hit> = None;
        for face in faces {
            if let Some(h) = self.in_face(face.orientation(), face.plane_coordinate(), ray) {
                closest = match closest {
                    Some(hc) if h.t < hc.t => { Some(h) }
                    None => { Some(h) }
                    _ => { closest }
                };
            }
        }

        return closest;
    }
}

impl Object for Cube {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        if let Some(h) = self.intersect_face(ray) {
            return Some(h);
        }

        None
    }
}
