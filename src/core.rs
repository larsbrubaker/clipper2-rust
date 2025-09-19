//! Core types and structures for Clipper2
//!
//! Direct port from clipper.core.h
//! This module contains the fundamental data types and basic operations

use num_traits::{Float, Num, Zero};
use std::fmt::{Debug, Display};

/// Fill rule determines how polygons with self-intersections are filled
/// Direct port from clipper.core.h line 108
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub enum FillRule {
    /// Even-odd fill rule (also known as Alternate)
    #[default]
    EvenOdd,
    /// Non-zero fill rule (also known as Winding)
    NonZero,
    /// Positive fill rule
    Positive,
    /// Negative fill rule
    Negative,
}

/// Exception type for Clipper2 errors
/// Direct port from clipper.core.h line 27
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clipper2Exception {
    description: String,
}

impl Clipper2Exception {
    pub fn new(description: &str) -> Self {
        Self {
            description: description.to_string(),
        }
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

impl Display for Clipper2Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clipper2Exception: {}", self.description)
    }
}

impl std::error::Error for Clipper2Exception {}

/// Handle errors by throwing appropriate exceptions
/// Direct port from clipper.core.h line 73
pub fn do_error(error_code: i32) -> Result<(), Clipper2Exception> {
    use errors::*;

    match error_code {
        PRECISION_ERROR_I => Err(Clipper2Exception::new(PRECISION_ERROR)),
        SCALE_ERROR_I => Err(Clipper2Exception::new(SCALE_ERROR)),
        NON_PAIR_ERROR_I => Err(Clipper2Exception::new(NON_PAIR_ERROR)),
        UNDEFINED_ERROR_I => Err(Clipper2Exception::new(UNDEFINED_ERROR)),
        RANGE_ERROR_I => Err(Clipper2Exception::new(RANGE_ERROR)),
        _ => Err(Clipper2Exception::new("Unknown error")),
    }
}

/// Constants matching C++ implementation
/// Direct port from clipper.core.h line 55-71
pub mod constants {
    /// PI constant
    pub const PI: f64 = std::f64::consts::PI;

    /// Maximum decimal precision for clipper operations
    pub const CLIPPER2_MAX_DEC_PRECISION: i32 = 8;

    /// Maximum coordinate value (INT64_MAX >> 2)
    pub const MAX_COORD: i64 = i64::MAX >> 2;
    /// Minimum coordinate value  
    pub const MIN_COORD: i64 = -MAX_COORD;
    /// Invalid coordinate sentinel
    pub const INVALID: i64 = i64::MAX;
    /// Maximum coordinate as double
    pub const MAX_COORD_D: f64 = MAX_COORD as f64;
    /// Minimum coordinate as double
    pub const MIN_COORD_D: f64 = MIN_COORD as f64;
    /// Maximum double value
    pub const MAX_DBL: f64 = f64::MAX;
}

/// Error constants matching C++ implementation
pub mod errors {
    /// Precision exceeds the permitted range
    pub const PRECISION_ERROR: &str = "Precision exceeds the permitted range";
    /// Values exceed permitted range
    pub const RANGE_ERROR: &str = "Values exceed permitted range";
    /// Invalid scale (either 0 or too large)
    pub const SCALE_ERROR: &str = "Invalid scale (either 0 or too large)";
    /// There must be 2 values for each coordinate
    pub const NON_PAIR_ERROR: &str = "There must be 2 values for each coordinate";
    /// There is an undefined error in Clipper2
    pub const UNDEFINED_ERROR: &str = "There is an undefined error in Clipper2";

    /// Error codes (2^n) - non-fatal
    pub const PRECISION_ERROR_I: i32 = 1;
    /// Error codes (2^n) - non-fatal  
    pub const SCALE_ERROR_I: i32 = 2;
    /// Error codes (2^n) - non-fatal
    pub const NON_PAIR_ERROR_I: i32 = 4;
    /// Error codes (2^n) - fatal
    pub const UNDEFINED_ERROR_I: i32 = 32;
    /// Error codes (2^n)
    pub const RANGE_ERROR_I: i32 = 64;
}

/// 2D point with generic numeric type
/// Direct port from clipper.core.h line 117
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T>
where
    T: Num + Copy,
{
    /// Create a new point
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Create a zero point
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }
}

impl<T> Point<T>
where
    T: Num + Copy,
{
    /// Add two points
    pub fn add_point(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    /// Subtract two points
    pub fn sub_point(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    /// Negate a point
    pub fn negate(self) -> Self {
        Self {
            x: T::zero() - self.x,
            y: T::zero() - self.y,
        }
    }
}

impl<T> Point<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Scale a point by a floating-point factor  
    pub fn scale<F>(self, scale: F) -> Point<F>
    where
        F: Float,
        T: Into<F>,
    {
        Point {
            x: self.x.into() * scale,
            y: self.y.into() * scale,
        }
    }
}

// Operator overloads matching C++
impl<T> std::ops::Add for Point<T>
where
    T: Num + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add_point(rhs)
    }
}

impl<T> std::ops::Sub for Point<T>
where
    T: Num + Copy,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.sub_point(rhs)
    }
}

impl<T> std::ops::Neg for Point<T>
where
    T: Num + Copy,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.negate()
    }
}

/// Rectangle with generic numeric type
/// Direct port from clipper.core.h line 295
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl<T> Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    /// Create a new rectangle
    pub fn new(left: T, top: T, right: T, bottom: T) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Create a rectangle, valid by default or invalid if specified
    /// Direct port from clipper.core.h line 307
    pub fn new_with_validity(is_valid: bool) -> Self
    where
        T: num_traits::Bounded,
    {
        if is_valid {
            Self {
                left: T::zero(),
                top: T::zero(),
                right: T::zero(),
                bottom: T::zero(),
            }
        } else {
            Self {
                left: T::max_value(),
                top: T::max_value(),
                right: T::min_value(),
                bottom: T::min_value(),
            }
        }
    }

    /// Create an invalid rectangle
    /// Direct port from clipper.core.h line 320
    pub fn invalid() -> Self
    where
        T: num_traits::Bounded,
    {
        Self {
            left: T::max_value(),
            top: T::max_value(),
            right: T::min_value(),
            bottom: T::min_value(),
        }
    }

    /// Get midpoint of rectangle
    /// Direct port from clipper.core.h line 336
    pub fn mid_point(&self) -> Point<T> {
        Point {
            x: (self.left + self.right) / (T::one() + T::one()),
            y: (self.top + self.bottom) / (T::one() + T::one()),
        }
    }

    /// Convert rectangle to path (clockwise from top-left)
    /// Direct port from clipper.core.h line 341
    pub fn as_path(&self) -> Path<T> {
        vec![
            Point::new(self.left, self.top),
            Point::new(self.right, self.top),
            Point::new(self.right, self.bottom),
            Point::new(self.left, self.bottom),
        ]
    }

    /// Check if point is contained within rectangle (exclusive bounds)
    /// Direct port from clipper.core.h line 352
    pub fn contains_point(&self, pt: &Point<T>) -> bool {
        pt.x > self.left && pt.x < self.right && pt.y > self.top && pt.y < self.bottom
    }

    /// Check if another rectangle is fully contained within this rectangle
    /// Direct port from clipper.core.h line 357
    pub fn contains_rect(&self, rec: &Rect<T>) -> bool {
        rec.left >= self.left
            && rec.right <= self.right
            && rec.top >= self.top
            && rec.bottom <= self.bottom
    }

    /// Check if this rectangle intersects with another
    /// Direct port from clipper.core.h line 372
    pub fn intersects(&self, rec: &Rect<T>) -> bool {
        let max_left = if self.left > rec.left {
            self.left
        } else {
            rec.left
        };
        let min_right = if self.right < rec.right {
            self.right
        } else {
            rec.right
        };
        let max_top = if self.top > rec.top {
            self.top
        } else {
            rec.top
        };
        let min_bottom = if self.bottom < rec.bottom {
            self.bottom
        } else {
            rec.bottom
        };

        max_left <= min_right && max_top <= min_bottom
    }

    /// Check if rectangle is valid (not using max sentinel values)
    /// Direct port from clipper.core.h line 329
    pub fn is_valid(&self) -> bool
    where
        T: num_traits::Bounded + PartialEq,
    {
        self.left != T::max_value()
    }

    /// Get width of rectangle
    pub fn width(&self) -> T {
        self.right - self.left
    }

    /// Get height of rectangle  
    pub fn height(&self) -> T {
        self.bottom - self.top
    }

    /// Set width, adjusting right edge
    pub fn set_width(&mut self, width: T) {
        self.right = self.left + width;
    }

    /// Set height, adjusting bottom edge
    pub fn set_height(&mut self, height: T) {
        self.bottom = self.top + height;
    }

    /// Check if rectangle is empty
    pub fn is_empty(&self) -> bool {
        self.left >= self.right || self.top >= self.bottom
    }
}

impl<T> Rect<T>
where
    T: Float + Copy,
{
    /// Scale rectangle by floating-point factor
    pub fn scale(&mut self, scale: T) {
        self.left = self.left * scale;
        self.top = self.top * scale;
        self.right = self.right * scale;
        self.bottom = self.bottom * scale;
    }
}

// Implement PartialEq for Rect to match C++ operator==
// Direct port from clipper.core.h line 378
impl<T> PartialEq for Rect<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.left == other.left
            && self.right == other.right
            && self.top == other.top
            && self.bottom == other.bottom
    }
}

// Implement += operator for Rect (union operation)
// Direct port from clipper.core.h line 383
impl<T> std::ops::AddAssign for Rect<T>
where
    T: Num + Copy + PartialOrd,
{
    fn add_assign(&mut self, other: Self) {
        self.left = if self.left < other.left {
            self.left
        } else {
            other.left
        };
        self.top = if self.top < other.top {
            self.top
        } else {
            other.top
        };
        self.right = if self.right > other.right {
            self.right
        } else {
            other.right
        };
        self.bottom = if self.bottom > other.bottom {
            self.bottom
        } else {
            other.bottom
        };
    }
}

// Type aliases matching C++ implementation
pub type Point64 = Point<i64>;
pub type PointD = Point<f64>;
pub type Rect64 = Rect<i64>;
pub type RectD = Rect<f64>;

/// Vector of points forming a path
pub type Path<T> = Vec<Point<T>>;
pub type Path64 = Path<i64>;
pub type PathD = Path<f64>;

/// Vector of paths
pub type Paths<T> = Vec<Path<T>>;
pub type Paths64 = Paths<i64>;
pub type PathsD = Paths<f64>;

/// Invalid point constants
pub const INVALID_POINT64: Point64 = Point64 {
    x: i64::MAX,
    y: i64::MAX,
};

pub const INVALID_POINTD: PointD = PointD {
    x: f64::MAX,
    y: f64::MAX,
};

/// Calculate midpoint between two points
/// Direct port from clipper.core.h line 278
#[inline]
pub fn mid_point<T>(p1: Point<T>, p2: Point<T>) -> Point<T>
where
    T: Num + Copy,
{
    Point {
        x: (p1.x + p2.x) / (T::one() + T::one()),
        y: (p1.y + p2.y) / (T::one() + T::one()),
    }
}

/// Helper trait for converting to f64 - matching C++ static_cast<double> behavior
pub trait ToF64 {
    fn to_f64(self) -> f64;
}

impl ToF64 for i64 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl ToF64 for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}

impl ToF64 for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

/// Calculate cross product of two vectors formed by three points  
/// Direct port from clipper.core.h line 810
#[inline]
pub fn cross_product_three_points<T>(pt1: Point<T>, pt2: Point<T>, pt3: Point<T>) -> f64
where
    T: Copy + ToF64,
{
    let pt1_x = pt1.x.to_f64();
    let pt1_y = pt1.y.to_f64();
    let pt2_x = pt2.x.to_f64();
    let pt2_y = pt2.y.to_f64();
    let pt3_x = pt3.x.to_f64();
    let pt3_y = pt3.y.to_f64();

    (pt2_x - pt1_x) * (pt3_y - pt2_y) - (pt2_y - pt1_y) * (pt3_x - pt2_x)
}

/// Calculate cross product of two vectors
/// Direct port from clipper.core.h line 816
#[inline]
pub fn cross_product_two_vectors<T>(vec1: Point<T>, vec2: Point<T>) -> f64
where
    T: Copy + ToF64,
{
    let vec1_x = vec1.x.to_f64();
    let vec1_y = vec1.y.to_f64();
    let vec2_x = vec2.x.to_f64();
    let vec2_y = vec2.y.to_f64();

    vec1_y * vec2_x - vec2_y * vec1_x
}

/// Calculate dot product of two vectors formed by three points
/// Direct port from clipper.core.h line 822
#[inline]
pub fn dot_product_three_points<T>(pt1: Point<T>, pt2: Point<T>, pt3: Point<T>) -> f64
where
    T: Copy + ToF64,
{
    let pt1_x = pt1.x.to_f64();
    let pt1_y = pt1.y.to_f64();
    let pt2_x = pt2.x.to_f64();
    let pt2_y = pt2.y.to_f64();
    let pt3_x = pt3.x.to_f64();
    let pt3_y = pt3.y.to_f64();

    (pt2_x - pt1_x) * (pt3_x - pt2_x) + (pt2_y - pt1_y) * (pt3_y - pt2_y)
}

/// Calculate dot product of two vectors
/// Direct port from clipper.core.h line 828
#[inline]
pub fn dot_product_two_vectors<T>(vec1: Point<T>, vec2: Point<T>) -> f64
where
    T: Copy + ToF64,
{
    let vec1_x = vec1.x.to_f64();
    let vec1_y = vec1.y.to_f64();
    let vec2_x = vec2.x.to_f64();
    let vec2_y = vec2.y.to_f64();

    vec1_x * vec2_x + vec1_y * vec2_y
}

/// Helper for returning -1, 0, or 1 based on sign
/// Direct port from clipper.core.h line 697  
#[inline]
pub fn tri_sign(x: i64) -> i32 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else {
        0
    }
}

/// 128-bit unsigned integer struct for high-precision multiplication
/// Direct port from clipper.core.h line 685
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UInt128Struct {
    pub lo: u64,
    pub hi: u64,
}

/// Multiply two 64-bit unsigned integers to get 128-bit result
/// Direct port from clipper.core.h line 690
#[inline]
pub fn multiply_u64(a: u64, b: u64) -> UInt128Struct {
    // Lambdas from C++: lo = x & 0xFFFFFFFF, hi = x >> 32
    let lo = |x: u64| -> u64 { x & 0xFFFFFFFF };
    let hi = |x: u64| -> u64 { x >> 32 };

    let x1 = lo(a) * lo(b);
    let x2 = hi(a) * lo(b) + hi(x1);
    let x3 = lo(a) * hi(b) + lo(x2);
    let lobits = lo(x3) << 32 | lo(x1);
    let hibits = hi(a) * hi(b) + hi(x2) + hi(x3);

    UInt128Struct {
        lo: lobits,
        hi: hibits,
    }
}

/// Check if products a*b and c*d are equal using exact 128-bit arithmetic
/// Direct port from clipper.core.h line 703
#[inline]
pub fn products_are_equal(a: i64, b: i64, c: i64, d: i64) -> bool {
    // For 128-bit capable systems, use i128 for simplicity
    #[cfg(target_pointer_width = "64")]
    {
        let ab = a as i128 * b as i128;
        let cd = c as i128 * d as i128;
        ab == cd
    }

    // For other systems or if we want exact C++ behavior, use the manual implementation
    #[cfg(not(target_pointer_width = "64"))]
    {
        // Convert to unsigned for overflow calculations
        let abs_a = a.unsigned_abs();
        let abs_b = b.unsigned_abs();
        let abs_c = c.unsigned_abs();
        let abs_d = d.unsigned_abs();

        let ab = multiply_u64(abs_a, abs_b);
        let cd = multiply_u64(abs_c, abs_d);

        // Calculate signs - important to differentiate 0 values
        let sign_ab = tri_sign(a) * tri_sign(b);
        let sign_cd = tri_sign(c) * tri_sign(d);

        ab == cd && sign_ab == sign_cd
    }
}

/// Strip duplicate consecutive points from a path
/// Direct port from clipper.core.h line 658
#[inline]
pub fn strip_duplicates_path<T>(path: &mut Path<T>, is_closed_path: bool)
where
    T: PartialEq + Clone,
{
    // Use stable dedup to remove consecutive duplicates
    path.dedup();

    // For closed paths, also remove duplicates between last and first points
    if is_closed_path {
        while path.len() > 1 && path.last() == path.first() {
            path.pop();
        }
    }
}

/// Strip duplicate consecutive points from multiple paths
/// Direct port from clipper.core.h line 670
#[inline]
pub fn strip_duplicates_paths<T>(paths: &mut Paths<T>, is_closed_path: bool)
where
    T: PartialEq + Clone,
{
    for path in paths.iter_mut() {
        strip_duplicates_path(path, is_closed_path);
    }
}

/// Check if precision is within acceptable range and adjust if needed
/// Direct port from clipper.core.h line 682
#[inline]
pub fn check_precision_range(precision: &mut i32, error_code: &mut i32) {
    use constants::CLIPPER2_MAX_DEC_PRECISION;
    use errors::PRECISION_ERROR_I;

    if *precision >= -CLIPPER2_MAX_DEC_PRECISION && *precision <= CLIPPER2_MAX_DEC_PRECISION {
        return;
    }

    *error_code |= PRECISION_ERROR_I; // non-fatal error

    // In Rust, we return the error instead of calling DoError with exceptions
    // This matches the C++ behavior when exceptions are disabled

    *precision = if *precision > 0 {
        CLIPPER2_MAX_DEC_PRECISION
    } else {
        -CLIPPER2_MAX_DEC_PRECISION
    };
}

/// Check precision range without error code (convenience function)
/// Direct port from clipper.core.h line 691
#[inline]
pub fn check_precision_range_simple(precision: &mut i32) {
    let mut error_code = 0;
    check_precision_range(precision, &mut error_code);
}

/// Calculate the bounding rectangle of a path
/// Direct port from clipper.core.h line 432
#[inline]
pub fn get_bounds_path<T>(path: &Path<T>) -> Rect<T>
where
    T: Copy + PartialOrd + num_traits::Bounded + num_traits::Num,
{
    let mut xmin = T::max_value();
    let mut ymin = T::max_value();
    let mut xmax = T::min_value();
    let mut ymax = T::min_value();

    for p in path {
        if p.x < xmin {
            xmin = p.x;
        }
        if p.x > xmax {
            xmax = p.x;
        }
        if p.y < ymin {
            ymin = p.y;
        }
        if p.y > ymax {
            ymax = p.y;
        }
    }

    Rect::new(xmin, ymin, xmax, ymax)
}

/// Calculate the bounding rectangle of multiple paths
/// Direct port from clipper.core.h line 449
#[inline]
pub fn get_bounds_paths<T>(paths: &Paths<T>) -> Rect<T>
where
    T: Copy + PartialOrd + num_traits::Bounded + num_traits::Num,
{
    let mut xmin = T::max_value();
    let mut ymin = T::max_value();
    let mut xmax = T::min_value();
    let mut ymax = T::min_value();

    for path in paths {
        for p in path {
            if p.x < xmin {
                xmin = p.x;
            }
            if p.x > xmax {
                xmax = p.x;
            }
            if p.y < ymin {
                ymin = p.y;
            }
            if p.y > ymax {
                ymax = p.y;
            }
        }
    }

    Rect::new(xmin, ymin, xmax, ymax)
}

/// Calculate the bounding rectangle of a path with type conversion
/// Direct port from clipper.core.h line 467
#[inline]
pub fn get_bounds_path_convert<T, T2>(path: &Path<T2>) -> Rect<T>
where
    T: Copy + PartialOrd + num_traits::Bounded + num_traits::Num,
    T2: Copy + Into<T>,
{
    let mut xmin = T::max_value();
    let mut ymin = T::max_value();
    let mut xmax = T::min_value();
    let mut ymax = T::min_value();

    for p in path {
        let x: T = p.x.into();
        let y: T = p.y.into();
        if x < xmin {
            xmin = x;
        }
        if x > xmax {
            xmax = x;
        }
        if y < ymin {
            ymin = y;
        }
        if y > ymax {
            ymax = y;
        }
    }

    Rect::new(xmin, ymin, xmax, ymax)
}

/// Calculate the bounding rectangle of multiple paths with type conversion
/// Direct port from clipper.core.h line 484
#[inline]
pub fn get_bounds_paths_convert<T, T2>(paths: &Paths<T2>) -> Rect<T>
where
    T: Copy + PartialOrd + num_traits::Bounded + num_traits::Num,
    T2: Copy + Into<T>,
{
    let mut xmin = T::max_value();
    let mut ymin = T::max_value();
    let mut xmax = T::min_value();
    let mut ymax = T::min_value();

    for path in paths {
        for p in path {
            let x: T = p.x.into();
            let y: T = p.y.into();
            if x < xmin {
                xmin = x;
            }
            if x > xmax {
                xmax = x;
            }
            if y < ymin {
                ymin = y;
            }
            if y > ymax {
                ymax = y;
            }
        }
    }

    Rect::new(xmin, ymin, xmax, ymax)
}

/// Square a value (matches C++ template<typename T> inline double Sqr(T val))
/// Direct port from clipper.core.h line 611
#[inline]
pub fn sqr<T>(val: T) -> f64
where
    T: ToF64,
{
    let d = val.to_f64();
    d * d
}

/// Calculate squared distance between two points
/// Direct port from clipper.core.h line 834
#[inline]
pub fn distance_sqr<T>(pt1: Point<T>, pt2: Point<T>) -> f64
where
    T: Copy + ToF64,
{
    sqr(pt1.x.to_f64() - pt2.x.to_f64()) + sqr(pt1.y.to_f64() - pt2.y.to_f64())
}

/// Calculate squared perpendicular distance from point to line
/// Direct port from clipper.core.h line 840
#[inline]
pub fn perpendicular_distance_from_line_sqr<T>(
    pt: Point<T>,
    line1: Point<T>,
    line2: Point<T>,
) -> f64
where
    T: Copy + ToF64,
{
    let a = pt.x.to_f64() - line1.x.to_f64();
    let b = pt.y.to_f64() - line1.y.to_f64();
    let c = line2.x.to_f64() - line1.x.to_f64();
    let d = line2.y.to_f64() - line1.y.to_f64();

    if c == 0.0 && d == 0.0 {
        return 0.0;
    }

    sqr(a * d - c * b) / (c * c + d * d)
}

/// Calculate area of a polygon path using the shoelace formula
/// Direct port from clipper.core.h line 854
#[inline]
pub fn area<T>(path: &Path<T>) -> f64
where
    T: Copy + ToF64,
{
    let cnt = path.len();
    if cnt < 3 {
        return 0.0;
    }

    // Use the standard shoelace formula for now - matches what C++ should produce
    let mut area = 0.0;
    for i in 0..cnt {
        let j = (i + 1) % cnt;
        let xi = path[i].x.to_f64();
        let yi = path[i].y.to_f64();
        let xj = path[j].x.to_f64();
        let yj = path[j].y.to_f64();

        area += xi * yj - xj * yi;
    }

    area * 0.5
}

/// Calculate total area of multiple polygon paths
/// Direct port from clipper.core.h line 874
#[inline]
pub fn area_paths<T>(paths: &Paths<T>) -> f64
where
    T: Copy + ToF64,
{
    let mut total_area = 0.0;
    for path in paths {
        total_area += area(path);
    }
    total_area
}

/// Test if a polygon has positive orientation (counterclockwise)
/// Direct port from clipper.core.h line 886
#[inline]
pub fn is_positive<T>(poly: &Path<T>) -> bool
where
    T: Copy + ToF64,
{
    area(poly) >= 0.0
}

// Include tests from separate file
#[cfg(test)]
#[path = "core_tests.rs"]
mod tests;
