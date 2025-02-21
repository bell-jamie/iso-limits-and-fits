use std::f64::consts::PI;

use super::utils::decimals;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Segment {
    pub p1: Point,
    pub p2: Point,
    pub m: f64,
    pub c: f64,
}

impl Segment {
    pub fn from(p1: &Point, p2: &Point) -> Self {
        let m = (p2.y - p1.y) / (p2.x - p1.x);
        let c = p1.y - m * p1.x;

        Self {
            p1: p1.clone(),
            p2: p2.clone(),
            m,
            c,
        }
    }

    pub fn new(p1: [f64; 2], p2: [f64; 2]) -> Self {
        Segment::from(&Point::from(p1), &Point::from(p2))
    }

    pub fn from_point_angle(p1: &Point, angle: f64) -> Self {
        let p2 = Point::new(p1.x * angle.cos(), p1.y * angle.sin());
        Segment::from(&p1, &p2)
    }

    pub fn gradient(&self) -> f64 {
        (self.p2.y - self.p1.y) / (self.p2.x - self.p1.x)
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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Rectangle {
    pub p1: Point,
    pub p2: Point,
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

        Self {
            p1: p1.clone(),
            p2: p2.clone(),
            x1,
            x2,
            y1,
            y2,
        }
    }

    pub fn new(p1: [f64; 2], p2: [f64; 2]) -> Self {
        Rectangle::from(&Point::from(p1), &Point::from(p2))
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

    pub fn intersections(&self, s: &Segment) -> Option<Vec<Point>> {
        let mut intersections = Vec::new();
        let x1 = self.x1.min(self.x2);
        let x2 = self.x1.max(self.x2);
        let y1 = self.y1.min(self.y2);
        let y2 = self.y1.max(self.y2);
        let bounds_x = x1..x2;
        let bounds_y = y1..y2;

        let ints_x = [(s.find_x(y1), y1), (s.find_x(y2), y2)];
        let ints_y = [(x1, s.find_y(x1)), (x2, s.find_y(x2))];

        for (x, y) in ints_x {
            if bounds_x.contains(&x) {
                intersections.push(Point::new(x, y));
            }
        }

        for (x, y) in ints_y {
            if bounds_y.contains(&y) {
                intersections.push(Point::new(x, y));
            }
        }

        // I'm not sure that the option here is actually needed
        if intersections.is_empty() {
            None
        } else {
            Some(intersections)
        }
    }
}

pub fn xy_bounds_check(x_min: f64, x_max: f64, y_min: f64, y_max: f64, p: Point) -> bool {
    p.x > x_min && p.x < x_max && p.y > y_min && p.y < y_max
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
        let s1 = Segment::from(&p3, &p4);

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
}
