use crate::colour::Colour;
use crate::object::Object;
use crate::ray::Ray;
use crate::vector::Vector;

pub struct Cube {
    pub c: Vector<f64, 3>, // Centre
    pub d: f64,  // Distance from centre to face centres
    pub colour: Colour,
}

impl Object for Cube {
    fn intersect(&self, ray: &Ray) -> Option<(Vector<f64, 3>, f64)>  {
        // Front (-1, -1, -1) to (1, 1, -1)
        // t = (c_z - s_z + -1 * d_bz) / d_rz
        let t = (self.c.z() - ray.s.z() + -1.0 * self.d) / ray.d.z();
        let h = ray.at(t);
        if h.x() < self.c.x() + -1.0 * self.d || h.x() > self.c.x() + 1.0 * self.d {
            return None
        }
        if h.y() < self.c.y() + -1.0 * self.d || h.y() > self.c.y() + 1.0 * self.d {
            return None
        }

        // Back  (-1, -1, 1) to (1, 1, 1)
        // Left (-1, -1, -1) to (-1, 1, 1)
        // Right (1, -1, -1) to (1, 1, 1)
        // Bottom (-1, -1, -1) to (1, -1, 1)
        // Top (-1, 1, -1) to (1, 1, 1)




        Some((h, t))
    }

    fn normal(&self, at: &Vector<f64, 3>) -> Vector<f64, 3> {
        Vector::new([0.0, 0.0, -1.0])
    }

    fn material(&self) -> Colour {
        self.colour
    }
}