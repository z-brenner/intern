//! Points, rotated rectangles, and the quadrilaterals a text line is cut
//! out of.
//!
//! Detection finds each line as a blob on a probability map, and the
//! rectangle it is read through is the smallest one, at any angle, that
//! holds the blob. A line on a page scanned three degrees off is then cut
//! out along its own slope rather than as an upright box that also catches
//! the lines above and below it.

/// A position in image pixels, origin top left, y down.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn minus(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    fn distance(self, other: Self) -> f32 {
        let delta = self.minus(other);
        (delta.x * delta.x + delta.y * delta.y).sqrt()
    }

    /// This point turned by `radians` about `centre`, counter-clockwise on
    /// screen for a positive angle (y points down).
    pub fn rotated_about(self, centre: Self, radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        let offset = self.minus(centre);
        Self::new(
            centre.x + offset.x * cos + offset.y * sin,
            centre.y - offset.x * sin + offset.y * cos,
        )
    }
}

fn cross(origin: Point, a: Point, b: Point) -> f32 {
    (a.x - origin.x) * (b.y - origin.y) - (a.y - origin.y) * (b.x - origin.x)
}

/// The convex hull of `points`, by Andrew's monotone chain. Duplicate and
/// collinear points are dropped.
pub fn convex_hull(points: &[Point]) -> Vec<Point> {
    let mut sorted = points.to_vec();
    sorted.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    sorted.dedup();
    if sorted.len() < 3 {
        return sorted;
    }
    let mut hull: Vec<Point> = Vec::with_capacity(sorted.len() * 2);
    for &point in &sorted {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0 {
            hull.pop();
        }
        hull.push(point);
    }
    let lower = hull.len() + 1;
    for &point in sorted.iter().rev().skip(1) {
        while hull.len() >= lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0
        {
            hull.pop();
        }
        hull.push(point);
    }
    hull.pop();
    hull
}

/// A rectangle at any angle: its centre, its two side lengths, and the
/// direction of its `width` side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RotatedRect {
    pub centre: Point,
    pub width: f32,
    pub height: f32,
    /// Direction of the `width` side, as a unit vector.
    pub axis: Point,
}

impl RotatedRect {
    /// The four corners, in no particular order.
    pub fn corners(&self) -> [Point; 4] {
        let (ux, uy) = (
            self.axis.x * self.width / 2.0,
            self.axis.y * self.width / 2.0,
        );
        let (nx, ny) = (
            -self.axis.y * self.height / 2.0,
            self.axis.x * self.height / 2.0,
        );
        let c = self.centre;
        [
            Point::new(c.x - ux - nx, c.y - uy - ny),
            Point::new(c.x + ux - nx, c.y + uy - ny),
            Point::new(c.x + ux + nx, c.y + uy + ny),
            Point::new(c.x - ux + nx, c.y - uy + ny),
        ]
    }

    pub fn shorter_side(&self) -> f32 {
        self.width.min(self.height)
    }

    /// The rectangle grown by `distance` on every side.
    ///
    /// Detection marks the core of each line, narrower than its ink; the
    /// reference post-processing grows each polygon back with a rounded
    /// offset and takes the smallest rectangle around the result. For a
    /// rectangle that is exactly this.
    pub fn grown(&self, distance: f32) -> Self {
        Self {
            width: self.width + 2.0 * distance,
            height: self.height + 2.0 * distance,
            ..*self
        }
    }
}

/// The smallest rectangle, at any angle, holding every point, by rotating
/// calipers over the hull: one side of the minimum rectangle always lies
/// along a hull edge. `None` for no points.
pub fn min_area_rect(points: &[Point]) -> Option<RotatedRect> {
    let hull = convex_hull(points);
    match hull.len() {
        0 => return None,
        1 => {
            return Some(RotatedRect {
                centre: hull[0],
                width: 0.0,
                height: 0.0,
                axis: Point::new(1.0, 0.0),
            });
        }
        _ => {}
    }
    let mut best: Option<(f32, RotatedRect)> = None;
    for index in 0..hull.len() {
        let start = hull[index];
        let end = hull[(index + 1) % hull.len()];
        let length = start.distance(end);
        if length <= f32::EPSILON {
            continue;
        }
        let axis = Point::new((end.x - start.x) / length, (end.y - start.y) / length);
        let normal = Point::new(-axis.y, axis.x);
        let (mut min_u, mut max_u, mut min_v, mut max_v) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for point in &hull {
            let offset = point.minus(start);
            let u = offset.x * axis.x + offset.y * axis.y;
            let v = offset.x * normal.x + offset.y * normal.y;
            min_u = min_u.min(u);
            max_u = max_u.max(u);
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }
        let area = (max_u - min_u) * (max_v - min_v);
        if best.as_ref().is_none_or(|(smallest, _)| area < *smallest) {
            let mid_u = (min_u + max_u) / 2.0;
            let mid_v = (min_v + max_v) / 2.0;
            best = Some((
                area,
                RotatedRect {
                    centre: Point::new(
                        start.x + axis.x * mid_u + normal.x * mid_v,
                        start.y + axis.y * mid_u + normal.y * mid_v,
                    ),
                    width: max_u - min_u,
                    height: max_v - min_v,
                    axis,
                },
            ));
        }
    }
    best.map(|(_, rect)| rect)
}

/// A text line's outline: four corners, clockwise from the top left as the
/// line reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad(pub [Point; 4]);

impl Quad {
    /// The corners of `rect` in reading order, the way the reference
    /// post-processing orders them: of the two leftmost corners the higher
    /// is the top left, of the two rightmost the higher is the top right.
    pub fn from_rect(rect: &RotatedRect) -> Self {
        let mut corners = rect.corners();
        corners.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        let (top_left, bottom_left) = if corners[1].y > corners[0].y {
            (corners[0], corners[1])
        } else {
            (corners[1], corners[0])
        };
        let (top_right, bottom_right) = if corners[3].y > corners[2].y {
            (corners[2], corners[3])
        } else {
            (corners[3], corners[2])
        };
        Self([top_left, top_right, bottom_right, bottom_left])
    }

    /// The length of the line: the longer of its top and bottom edges.
    pub fn width(&self) -> f32 {
        let [a, b, c, d] = self.0;
        a.distance(b).max(d.distance(c))
    }

    /// The height of the line: the longer of its left and right edges.
    pub fn height(&self) -> f32 {
        let [a, b, c, d] = self.0;
        a.distance(d).max(b.distance(c))
    }

    /// The slope of the top edge in degrees, positive when the line climbs
    /// to the right as a reader sees it.
    pub fn slope_degrees(&self) -> f32 {
        let [a, b, _, _] = self.0;
        (-(b.y - a.y)).atan2(b.x - a.x).to_degrees()
    }

    /// `[x0, y0, x1, y1]`, the upright box around the corners.
    pub fn bounds(&self) -> [f32; 4] {
        let xs = self.0.map(|point| point.x);
        let ys = self.0.map(|point| point.y);
        [
            xs.iter().copied().fold(f32::MAX, f32::min),
            ys.iter().copied().fold(f32::MAX, f32::min),
            xs.iter().copied().fold(f32::MIN, f32::max),
            ys.iter().copied().fold(f32::MIN, f32::max),
        ]
    }

    pub fn centre(&self) -> Point {
        let sum = self.0.iter().fold(Point::default(), |sum, point| {
            Point::new(sum.x + point.x, sum.y + point.y)
        });
        Point::new(sum.x / 4.0, sum.y / 4.0)
    }

    /// Whether `point` lies inside or on the outline, which is convex.
    pub fn contains(&self, point: Point) -> bool {
        let mut sign = 0.0_f32;
        for index in 0..4 {
            let turn = cross(self.0[index], self.0[(index + 1) % 4], point);
            if turn.abs() <= 1e-4 {
                continue;
            }
            if sign == 0.0 {
                sign = turn.signum();
            } else if turn.signum() != sign {
                return false;
            }
        }
        true
    }

    /// Every corner turned by `radians` about `centre`.
    pub fn rotated_about(&self, centre: Point, radians: f32) -> Self {
        Self(self.0.map(|point| point.rotated_about(centre, radians)))
    }

    /// Every corner scaled from one image's pixels to another's.
    pub fn scaled(&self, x: f32, y: f32) -> Self {
        Self(self.0.map(|point| Point::new(point.x * x, point.y * y)))
    }

    /// Every corner held inside a `width` by `height` image.
    pub fn clamped(&self, width: f32, height: f32) -> Self {
        Self(
            self.0
                .map(|point| Point::new(point.x.clamp(0.0, width), point.y.clamp(0.0, height))),
        )
    }

    /// The outline grown by `fraction` of the line's height above and below
    /// and by the same distance at each end.
    pub fn padded(&self, fraction: f32) -> Self {
        let [a, b, c, d] = self.0;
        let along = {
            let length = a.distance(b).max(f32::EPSILON);
            Point::new((b.x - a.x) / length, (b.y - a.y) / length)
        };
        let across = Point::new(-along.y, along.x);
        let pad = self.height() * fraction;
        let shift = |point: Point, u: f32, v: f32| {
            Point::new(
                point.x + along.x * u + across.x * v,
                point.y + along.y * u + across.y * v,
            )
        };
        Self([
            shift(a, -pad, -pad),
            shift(b, pad, -pad),
            shift(c, pad, pad),
            shift(d, -pad, pad),
        ])
    }
}

/// Reading order for detected lines: top to bottom, and left to right
/// within a row.
///
/// Lines are sorted by their top-left corner; then, as the reference
/// implementation does, a line that starts within `row_tolerance` pixels
/// of the one before it and to its left is moved ahead of it, so the cells
/// of one row are read left to right even when the right one sits a pixel
/// higher.
pub fn reading_order(quads: &[Quad], row_tolerance: f32) -> Vec<usize> {
    let mut order: Vec<usize> = (0..quads.len()).collect();
    order.sort_by(|&a, &b| {
        let (pa, pb) = (quads[a].0[0], quads[b].0[0]);
        pa.y.total_cmp(&pb.y).then(pa.x.total_cmp(&pb.x))
    });
    for index in 0..order.len().saturating_sub(1) {
        let mut position = index;
        loop {
            let (upper, lower) = (quads[order[position]].0[0], quads[order[position + 1]].0[0]);
            if (lower.y - upper.y).abs() < row_tolerance && lower.x < upper.x {
                order.swap(position, position + 1);
                if position == 0 {
                    break;
                }
                position -= 1;
            } else {
                break;
            }
        }
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn the_hull_drops_interior_and_collinear_points() {
        let points = [
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0),
            Point::new(4.0, 0.0),
            Point::new(4.0, 2.0),
            Point::new(0.0, 2.0),
            Point::new(1.0, 1.0),
            Point::new(4.0, 2.0),
        ];
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 4, "{hull:?}");
        for corner in [(0.0, 0.0), (4.0, 0.0), (4.0, 2.0), (0.0, 2.0)] {
            assert!(hull.contains(&Point::new(corner.0, corner.1)), "{hull:?}");
        }
    }

    #[test]
    fn the_minimum_rectangle_of_an_upright_box_is_that_box() {
        let points = [
            Point::new(10.0, 20.0),
            Point::new(110.0, 20.0),
            Point::new(110.0, 40.0),
            Point::new(10.0, 40.0),
        ];
        let rect = min_area_rect(&points).unwrap();
        assert!(close(rect.width.max(rect.height), 100.0), "{rect:?}");
        assert!(close(rect.shorter_side(), 20.0), "{rect:?}");
        assert!(close(rect.centre.x, 60.0) && close(rect.centre.y, 30.0));
        let quad = Quad::from_rect(&rect);
        assert_eq!(quad.0[0], Point::new(10.0, 20.0));
        assert_eq!(quad.0[1], Point::new(110.0, 20.0));
        assert_eq!(quad.0[2], Point::new(110.0, 40.0));
        assert_eq!(quad.0[3], Point::new(10.0, 40.0));
        assert!(close(quad.slope_degrees(), 0.0));
    }

    #[test]
    fn the_minimum_rectangle_follows_a_tilted_line() {
        // A 200 x 20 line climbing at five degrees.
        let rect = RotatedRect {
            centre: Point::new(300.0, 300.0),
            width: 200.0,
            height: 20.0,
            axis: Point::new(5_f32.to_radians().cos(), -(5_f32.to_radians().sin())),
        };
        let found = min_area_rect(&rect.corners()).unwrap();
        assert!(close(found.width * found.height, 4000.0), "{found:?}");
        let quad = Quad::from_rect(&found);
        assert!((quad.slope_degrees() - 5.0).abs() < 0.01, "{quad:?}");
        assert!(close(quad.width(), 200.0) && close(quad.height(), 20.0));
        assert!(quad.contains(Point::new(300.0, 300.0)));
        assert!(!quad.contains(Point::new(300.0, 330.0)));
    }

    #[test]
    fn growing_a_rectangle_adds_the_distance_on_every_side() {
        let rect = RotatedRect {
            centre: Point::new(0.0, 0.0),
            width: 10.0,
            height: 4.0,
            axis: Point::new(1.0, 0.0),
        };
        let grown = rect.grown(1.5);
        assert_eq!((grown.width, grown.height), (13.0, 7.0));
        assert_eq!(grown.centre, rect.centre);
    }

    #[test]
    fn rows_read_left_to_right_even_when_a_later_cell_sits_higher() {
        let line = |x: f32, y: f32| {
            Quad([
                Point::new(x, y),
                Point::new(x + 50.0, y),
                Point::new(x + 50.0, y + 10.0),
                Point::new(x, y + 10.0),
            ])
        };
        // The right cell of the first row is two pixels higher than the
        // left one; the second row is well below.
        let quads = [line(200.0, 100.0), line(10.0, 102.0), line(10.0, 140.0)];
        assert_eq!(reading_order(&quads, 10.0), vec![1, 0, 2]);
    }

    #[test]
    fn padding_grows_a_line_around_its_own_axis() {
        let quad = Quad([
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 20.0),
            Point::new(0.0, 20.0),
        ]);
        let padded = quad.padded(0.25);
        assert_eq!(padded.bounds(), [-5.0, -5.0, 105.0, 25.0]);
    }

    #[test]
    fn rotating_about_a_centre_keeps_distances() {
        let centre = Point::new(50.0, 50.0);
        let turned = Point::new(150.0, 50.0).rotated_about(centre, 90_f32.to_radians());
        // Counter-clockwise on screen: to the right becomes straight up.
        assert!(
            close(turned.x, 50.0) && close(turned.y, -50.0),
            "{turned:?}"
        );
    }
}
