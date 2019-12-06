use std::fs::File;
use std::io::{BufRead, BufReader};

extern crate geo;
extern crate line_intersection;

fn read_lines(filename: &str) -> impl Iterator<Item=String> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader.lines().map(|line| line.unwrap())
}

struct Point {
    x: i64,
    y: i64,
}

struct Wire {
    p0: Point,
    p1: Point,
}

fn parse_segment(s: &str) -> Point {
    let (dx, dy) = match s.chars().nth(0).unwrap() {
        'U' => (0, -1),
        'D' => (0, 1),
        'L' => (-1, 0),
        'R' => (1, 0),
        _ => (0, 0),
    };
    let magnitude = s[1..].parse::<i64>().unwrap();

    Point { x: dx * magnitude, y: dy * magnitude }
}

fn parse_wire(str: &str) -> Vec<Wire> {
    let segments = str.split(",").map(|x| parse_segment(x));
    let mut x = 0;
    let mut y = 0;
    let mut wire = Vec::new();
    for s in segments {
        wire.push(Wire {
            p0: Point {x: x, y: y},
            p1: Point {x: x + s.x, y: y + s.y}
        });
        x += s.x;
        y += s.y;
    }
    wire
}

fn get_intersection(wire: &Wire, other: &Wire) -> Option<Point> {
    use line_intersection::{LineInterval};

    let seg1 = LineInterval::line_segment(geo::Line {
        start: (wire.p0.x as f64, wire.p0.y as f64).into(),
        end: (wire.p1.x as f64, wire.p1.y as f64).into(),
    });

    let seg2 = LineInterval::line_segment(geo::Line {
        start: (other.p0.x as f64, other.p0.y as f64).into(),
        end: (other.p1.x as f64, other.p1.y as f64).into(),
    });

    let intersection = seg1.relate(&seg2).unique_intersection();
    match intersection {
        None => None,
        Some(geo::Point(geo::Coordinate {x, y})) => Some(Point {x: x.round() as i64, y: y.round() as i64}),
    }
}

fn main() {
    let lines: Vec<String> = read_lines("input.in").collect();

    let wire1 = parse_wire(&lines[0]);
    let wire2 = parse_wire(&lines[1]);

    let mut dist = 999999999;
    for a in &wire1 {
        for b in &wire2 {
            let isect = get_intersection(&a, &b);
            match isect {
                Some(p) => {
                    let new_dist = i64::abs(p.x) + i64::abs(p.y);
                    if new_dist < dist && new_dist > 0 {
                        dist = new_dist;
                    }
                },
                None => {},
            }
        }
    }
    println!("Best dist: {}", dist);
}
