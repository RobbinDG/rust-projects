use crate::colour::Colour;
use crate::object::Object;
use crate::ray::Ray;
use crate::vector::Vector;

const E: f64 = 0.0000001;
pub struct Cube {
    pub c: Vector<f64, 3>, // Centre
    pub d: f64,  // Distance from centre to face centres
    pub colour: Colour,
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
    fn in_face(&self, orientation: f64, i: usize, ray: &Ray) -> Option<(Vector<f64, 3>, f64)> {
        let t = (self.c[i] - ray.s[i] + orientation * self.d) / ray.d.z();
        let h = ray.at(t);

        let i1 = (i + 1) % 3;
        let i2 = (i + 2) % 3;
        let in_1 = h[i1] >= self.c[i1] + -1.0 * self.d - E && h[i1] <= self.c[i1] + 1.0 * self.d + E;
        let in_2 = h[i2] >= self.c[i2] + -1.0 * self.d - E && h[i2] <= self.c[i2] + 1.0 * self.d + E;
        if in_1 && in_2 { return Some((h, t)); }
        return None;
    }

    fn intersect_face(&self, ray: &Ray) -> Option<(Vector<f64, 3>, f64, CubeFace)> {
        let faces = vec![
            CubeFace::Front, // Front (-1, -1, -1) to (1, 1, -1)
            CubeFace::Back, // Back  (-1, -1, 1) to (1, 1, 1)
            CubeFace::Left, // Left (-1, -1, -1) to (-1, 1, 1)
            CubeFace::Right, // Right (1, -1, -1) to (1, 1, 1)
            CubeFace::Bottom, // Bottom (-1, -1, -1) to (1, -1, 1)
            CubeFace::Top, // Top (-1, 1, -1) to (1, 1, 1)
        ];

        // t = (c_z - s_z + -1 * d_bz) / d_rz
        let mut closest: Option<(Vector<f64, 3>, f64, CubeFace)> = None;
        for face in faces {
            if let Some((h, t)) = self.in_face(face.orientation(), face.plane_coordinate(), ray) {
                if let Some((_, tc, _)) = closest {
                    if t < tc {
                        closest = Some((h, t, face));
                    }
                } else {
                    closest = Some((h, t, face));
                }
            }
        }

        return closest;
    }
}

impl Object for Cube {
    fn intersect(&self, ray: &Ray) -> Option<(Vector<f64, 3>, f64)> {
        if let Some((h, t, _)) = self.intersect_face(ray) {
            return Some((h, t));
        }

        None
    }

    fn normal(&self, at: &Vector<f64, 3>) -> Vector<f64, 3> {
        Vector::new([0.0, 0.0, -1.0])
    }

    fn material(&self) -> Colour {
        self.colour
    }
}