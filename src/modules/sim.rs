// pub struct XYPoint {
//     x: i32,
//     y: i32,
// }

// pub struct XYMesh {
//     points: Vec<XYPoint>,
//     elements: Vec<usize>,
// }

pub struct RPoint {
    r: usize,
    t: usize,
}

pub struct QuadElement {
    p1: usize,
    p2: usize,
    p3: usize,
    p4: usize,
}

pub struct RQuadMesh {
    points: Vec<RPoint>,
    elements: Vec<QuadElement>,
}

impl RQuadMesh {
    fn create(
        elements_radial: usize,
        elements_tangential: usize,
        diameter_inner: f32,
        diameter_outer: f32,
    ) -> RQuadMesh {
        let points_radial = elements_radial + 1;
        let points_tangential = elements_tangential + 1;
        let points_total = points_radial * points_tangential;
        let elements_total = elements_radial * elements_tangential;

        let vec_elements = Vec::with_capacity(elements_total);
        let vec_points = Vec::with_capacity(points_total);

        for i in 0..points_tangential {
            for j in 0..points_radial {
                let theta = 2 * f32::PI * (i / points_tangential);
                let r = diameter_outer / 2.0;
                vec_points.push(RPoint {});
            }
        }
    }
}
