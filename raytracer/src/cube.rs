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
            (CubeFace::Front, -1.0, 2), // Front (-1, -1, -1) to (1, 1, -1)
            (CubeFace::Back, 1.0, 2), // Back  (-1, -1, 1) to (1, 1, 1)
            (CubeFace::Left, -1.0, 0), // Left (-1, -1, -1) to (-1, 1, 1)
            (CubeFace::Right, 1.0, 0), // Right (1, -1, -1) to (1, 1, 1)
            (CubeFace::Bottom, -1.0, 1), // Bottom (-1, -1, -1) to (1, -1, 1)
            (CubeFace::Top, 1.0, 1), // Top (-1, 1, -1) to (1, 1, 1)
        ];

        // t = (c_z - s_z + -1 * d_bz) / d_rz
        let mut closest: Option<(Vector<f64, 3>, f64, CubeFace)> = None;
        for (face, orientation, i) in faces {
             if let Some((h, t)) = self.in_face(orientation, i, ray){
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
        // Front (-1, -1, -1) to (1, 1, -1)
        // t = (c_z - s_z + -1 * d_bz) / d_rz
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