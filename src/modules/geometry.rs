use std::f64::consts::PI;

use super::utils::decimals;
use egui_plot::{Line, PlotPoint, PlotPoints, Polygon};
use serde::{Deserialize, Serialize};

const EPS: f64 = 1e-9; // Used to prevent divide-by-zero

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn from(p: [f64; 2]) -> Self {
        Self { x: p[0], y: p[1] }
    }

    pub fn to_arr(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    pub fn mirror_in_x(&mut self) {
        self.y = -self.y;
    }

    pub fn mirror_in_y(&mut self) {
        self.x = -self.x;
    }

    pub fn offset(&mut self, x: f64, y: f64) {
        self.x += x;
        self.y += y;
    }

    pub fn to_pp(&self) -> PlotPoint {
        PlotPoint {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Segment {
    pub p1: Point,
    pub p2: Point,
}

impl Segment {
    pub fn new(p1: &Point, p2: &Point) -> Self {
        Self {
            p1: p1.clone(),
            p2: p2.clone(),
        }
    }

    pub fn from_arr(p1: [f64; 2], p2: [f64; 2]) -> Self {
        Segment::new(&Point::from(p1), &Point::from(p2))
    }

    pub fn from_point_angle(p1: &Point, angle: f64) -> Self {
        let angle = angle.to_radians();
        let p2 = Point::new(p1.x + angle.cos(), p1.y + angle.sin());
        Segment::new(&p1, &p2)
    }

    pub fn from_x(x: f64) -> Self {
        Segment::new(&Point::new(x, -1.0), &Point::new(x, 1.0))
    }

    pub fn from_y(y: f64) -> Self {
        Segment::new(&Point::new(-1.0, y), &Point::new(1.0, y))
    }

    pub fn from_centre(centre: &Point, length: f64, angle: f64) -> Self {
        let mut horizontal = Segment::new(
            &Point::new(centre.x - length / 2.0, 0.0),
            &Point::new(centre.x + length / 2.0, 0.0),
        );
        horizontal.rotate(centre, angle);
        horizontal
    }

    pub fn rotate(&mut self, centre: &Point, angle: f64) {
        let mut points = vec![self.p1, self.p2];
        rotate_points(&mut points, centre, angle.to_radians());
        (self.p1, self.p2) = (points[0], points[1]);
    }

    pub fn offset(&mut self, x: f64, y: f64) {
        self.p1.offset(x, y);
        self.p2.offset(x, y);
    }

    pub fn offset_vector(&mut self, distance: f64, angle: f64) {
        let angle = angle.to_radians();
        let dx = distance * angle.cos();
        let dy = distance * angle.sin();
        self.offset(dx, dy);
    }

    pub fn gradient(&self) -> f64 {
        (self.p2.y - self.p1.y) / (self.p2.x - self.p1.x) + EPS
    }

    pub fn x_intersect(&self) -> f64 {
        -self.y_intersect() / self.gradient()
    }

    pub fn y_intersect(&self) -> f64 {
        self.p1.y - self.gradient() * self.p1.x
    }

    pub fn find_x(&self, y: f64) -> f64 {
        let m = self.gradient();
        let c = self.p1.y - m * self.p1.x;
        (y - c) / m
    }

    pub fn find_y(&self, x: f64) -> f64 {
        let m = self.gradient();
        let c = self.p1.y - m * self.p1.x;
        m * x + c
    }

    pub fn midpoint(&self) -> Point {
        Point::new((self.p1.x + self.p2.x) / 2.0, (self.p1.y + self.p2.y) / 2.0)
    }

    pub fn to_line(&self) -> Line {
        Line::new(PlotPoints::from(vec![self.p1.to_arr(), self.p2.to_arr()]))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
}

impl Rectangle {
    pub fn from(p1: &Point, p2: &Point) -> Self {
        let x1 = p1.x.min(p2.x);
        let x2 = p1.x.max(p2.x);
        let y1 = p1.y.min(p2.y);
        let y2 = p1.y.max(p2.y);

        Self { x1, x2, y1, y2 }
    }

    pub fn new(p1: [f64; 2], p2: [f64; 2]) -> Self {
        Rectangle::from(&Point::from(p1), &Point::from(p2))
    }

    pub fn scale(&mut self, scale: f64) {
        let cx = (self.x1 + self.x2) / 2.0; // X centre
        let cy = (self.y1 + self.y2) / 2.0; // Y centre

        self.x1 += (scale - 1.0) * (self.x1 - cx);
        self.y1 += (scale - 1.0) * (self.y1 - cy);
        self.x2 += (scale - 1.0) * (self.x2 - cx);
        self.y2 += (scale - 1.0) * (self.y2 - cy);
    }

    pub fn offset(&mut self, offset: f64) {
        self.x1 -= offset;
        self.y1 -= offset;
        self.x2 += offset;
        self.y2 += offset;
    }

    pub fn to_vec(&self) -> Vec<[f64; 2]> {
        vec![
            [self.x1, self.y2],
            [self.x1, self.y1],
            [self.x2, self.y1],
            [self.x2, self.y2],
            [self.x1, self.y2],
        ]
    }

    pub fn centre(&self) -> Point {
        let x = self.x1 + self.width() / 2.0;
        let y = self.y1 + self.height() / 2.0;
        Point { x, y }
    }

    pub fn width(&self) -> f64 {
        self.x2 - self.x1
    }

    pub fn height(&self) -> f64 {
        self.y2 - self.y1
    }

    pub fn to_poly(&self) -> Polygon {
        Polygon::new(PlotPoints::new(self.to_vec()))
    }

    pub fn intersections(&self, s: &Segment) -> Option<Vec<Point>> {
        let (x1, x2) = (self.x1.min(self.x2), self.x1.max(self.x2));
        let (y1, y2) = (self.y1.min(self.y2), self.y1.max(self.y2));

        let bounds_x = x1..=x2;
        let bounds_y = y1..=y2;

        let candidates = [
            (s.find_x(y1), y1),
            (s.find_x(y2), y2),
            (x1, s.find_y(x1)),
            (x2, s.find_y(x2)),
        ];

        let intersections: Vec<Point> = candidates
            .into_iter()
            .filter(|&(x, y)| bounds_x.contains(&x) && bounds_y.contains(&y))
            .map(|(x, y)| Point::new(x, y))
            .collect();

        if intersections.len() != 2 {
            None
        } else {
            Some(intersections)
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Path {
    pub points: Vec<Point>,
}

impl Path {
    // pub fn new(points: Vec<Point>) -> Self {
    //     Self { points }
    // }

    pub fn rotate(&mut self, centre: &Point, angle: f64) {
        rotate_points(&mut self.points, centre, angle.to_radians());
    }

    pub fn translate(&mut self, dx: f64, dy: f64) {
        translate_points(&mut self.points, dx, dy);
    }

    pub fn scale(&mut self, centre: &Point, scale: f64) {
        scale_points(&mut self.points, centre, scale);
    }

    pub fn mirror_in_x(&mut self) {
        mirror_points_in_x(&mut self.points);
    }

    pub fn mirror_in_y(&mut self) {
        mirror_points_in_y(&mut self.points);
    }

    pub fn to_line(&self) -> Line {
        Line::new(PlotPoints::from_iter(
            self.points.iter().map(|p| p.to_arr()),
        ))
    }

    /// This links the last point in the path to create a full polygon
    pub fn to_poly(&self) -> Polygon {
        Polygon::new(PlotPoints::from_iter(
            self.points
                .iter()
                .chain(vec![&self.points[0]])
                .map(|p| p.to_arr()),
        ))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Circle {
    pub centre: Point,
    pub radius: f64,
}

// impl Circle {

// }

pub fn xy_bounds_check(x_min: f64, x_max: f64, y_min: f64, y_max: f64, p: Point) -> bool {
    p.x > x_min && p.x < x_max && p.y > y_min && p.y < y_max
}

pub fn translate_points(points: &mut Vec<Point>, dx: f64, dy: f64) {
    for point in points.iter_mut() {
        point.x += dx;
        point.y += dy;
    }
}

pub fn scale_points(points: &mut Vec<Point>, centre: &Point, scale: f64) {
    for point in points.iter_mut() {
        point.x = ((point.x - centre.x) * scale) + centre.x;
        point.y = ((point.y - centre.y) * scale) + centre.y;
    }
}

pub fn rotate_points(points: &mut Vec<Point>, centre: &Point, angle: f64) {
    let (sin, cos) = (angle.sin(), angle.cos());

    for point in points.iter_mut() {
        let (x, y) = (point.x, point.y);
        point.x = centre.x + (x - centre.x) * cos - (y - centre.y) * sin;
        point.y = centre.y + (x - centre.x) * sin + (y - centre.y) * cos;
    }
}

pub fn mirror_points(points: &mut Vec<Point>, line: Segment) {
    let a = line.p2.y - line.p1.y;
    let b = line.p1.x - line.p2.x;
    let c = line.p2.x * line.p1.y - line.p1.x * line.p2.y;

    let denom = a * a + b * b;

    for point in points.iter_mut() {
        let d = (a * point.x + b * point.y + c) / denom;
        point.x = point.x - 2.0 * a * d;
        point.y = point.y - 2.0 * b * d;
    }
}

pub fn mirror_points_in_x(points: &mut Vec<Point>) {
    for point in points.iter_mut() {
        point.mirror_in_x();
    }
}

pub fn mirror_points_in_y(points: &mut Vec<Point>) {
    for point in points.iter_mut() {
        point.mirror_in_y();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_intersections() {
        let p1 = Point::new(10.0, 5.0);
        let p2 = Point::new(20.0, 10.0);
        let rect = Rectangle::from(&p1, &p2);

        let p3 = Point::new(50.0, 20.0);
        let p4 = Point::new(7.0, 5.0);
        let s1 = Segment::new(&p3, &p4);

        println!("Gradient = {}", s1.gradient());
        println!("Find where x = 12, y = {}", s1.find_y(12.0));

        if let Some(ints) = rect.intersections(&s1) {
            for (i, int) in ints.iter().enumerate() {
                println!("Intersection {i} - x: {}, y: {}", int.x, int.y);
            }
        } else {
            println!("No intersections");
        }

        assert!(1 == 1) // make this false to get outputs
    }

    #[test]
    fn test_mirror_point_through_line() {
        let mut points = vec![Point { x: 3.0, y: 4.0 }];
        let line_p1 = Point { x: 1.0, y: 1.0 };
        let line_p2 = Point { x: 4.0, y: 2.0 };

        mirror_points(&mut points, Segment::new(&line_p1, &line_p2));
        let mirrored = points[0];

        // Expected mirrored point is (4.4, -0.2)
        let expected = Point { x: 4.4, y: -0.2 };

        let epsilon = 1e-6; // Tolerance for floating-point comparison

        assert!(
            (mirrored.x - expected.x).abs() < epsilon,
            "X coordinate mismatch"
        );
        assert!(
            (mirrored.y - expected.y).abs() < epsilon,
            "Y coordinate mismatch"
        );
    }

    #[test]
    fn test_mirror_through_horizontal_line() {
        let mut points = vec![Point { x: 3.0, y: 4.0 }];
        let line_p1 = Point { x: 0.0, y: 2.0 };
        let line_p2 = Point { x: 5.0, y: 2.0 };

        mirror_points(&mut points, Segment::new(&line_p1, &line_p2));
        let mirrored = points[0];
        let expected = Point { x: 3.0, y: 0.0 };

        let epsilon = 1e-6;
        assert!(
            (mirrored.x - expected.x).abs() < epsilon,
            "X coordinate mismatch"
        );
        assert!(
            (mirrored.y - expected.y).abs() < epsilon,
            "Y coordinate mismatch"
        );
    }

    #[test]
    fn test_mirror_through_vertical_line() {
        let mut points = vec![Point { x: 3.0, y: 4.0 }];
        let line_p1 = Point { x: 2.0, y: 0.0 };
        let line_p2 = Point { x: 2.0, y: 5.0 };

        mirror_points(&mut points, Segment::new(&line_p1, &line_p2));
        let mirrored = points[0];
        let expected = Point { x: 1.0, y: 4.0 };

        let epsilon = 1e-6;
        assert!(
            (mirrored.x - expected.x).abs() < epsilon,
            "X coordinate mismatch"
        );
        assert!(
            (mirrored.y - expected.y).abs() < epsilon,
            "Y coordinate mismatch"
        );
    }
}
