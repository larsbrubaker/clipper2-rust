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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_rule_default() {
        assert_eq!(FillRule::default(), FillRule::EvenOdd);
    }

    #[test]
    fn test_fill_rule_variants() {
        let rules = [
            FillRule::EvenOdd,
            FillRule::NonZero,
            FillRule::Positive,
            FillRule::Negative,
        ];
        assert_eq!(rules.len(), 4);

        // Test each variant is unique
        for i in 0..rules.len() {
            for j in (i + 1)..rules.len() {
                assert_ne!(rules[i], rules[j]);
            }
        }
    }

    #[test]
    fn test_clipper2_exception() {
        let err = Clipper2Exception::new("test error");
        assert_eq!(err.description(), "test error");
        assert_eq!(err.to_string(), "Clipper2Exception: test error");
    }

    #[test]
    fn test_point_creation() {
        let p1 = Point::new(10i32, 20i32);
        assert_eq!(p1.x, 10);
        assert_eq!(p1.y, 20);

        let p2 = Point::<f64>::zero();
        assert_eq!(p2.x, 0.0);
        assert_eq!(p2.y, 0.0);
    }

    #[test]
    fn test_point_operations() {
        let p1 = Point::new(10i32, 20i32);
        let p2 = Point::new(5i32, 15i32);

        let sum = p1 + p2;
        assert_eq!(sum.x, 15);
        assert_eq!(sum.y, 35);

        let diff = p1 - p2;
        assert_eq!(diff.x, 5);
        assert_eq!(diff.y, 5);

        let neg = -p1;
        assert_eq!(neg.x, -10);
        assert_eq!(neg.y, -20);
    }

    #[test]
    fn test_point_scale() {
        let p1 = Point::new(10i32, 20i32);
        let scaled = p1.scale(2.5f64);
        assert_eq!(scaled.x, 25.0);
        assert_eq!(scaled.y, 50.0);
    }

    #[test]
    fn test_rect_creation() {
        let rect = Rect::new(0i32, 0i32, 100i32, 200i32);
        assert_eq!(rect.left, 0);
        assert_eq!(rect.top, 0);
        assert_eq!(rect.right, 100);
        assert_eq!(rect.bottom, 200);
    }

    #[test]
    fn test_rect_properties() {
        let rect = Rect::new(10i32, 20i32, 110i32, 220i32);

        assert!(rect.is_valid());
        assert_eq!(rect.width(), 100);
        assert_eq!(rect.height(), 200);
        assert!(!rect.is_empty());

        let empty_rect = Rect::new(100i32, 200i32, 50i32, 150i32);
        assert!(empty_rect.is_valid()); // Geometrically invalid but not sentinel invalid
        assert!(empty_rect.is_empty());
    }

    #[test]
    fn test_rect_modification() {
        let mut rect = Rect::new(0i32, 0i32, 50i32, 100i32);

        rect.set_width(200);
        assert_eq!(rect.right, 200);
        assert_eq!(rect.width(), 200);

        rect.set_height(300);
        assert_eq!(rect.bottom, 300);
        assert_eq!(rect.height(), 300);
    }

    #[test]
    fn test_rect_scale() {
        let mut rect = RectD::new(10.0, 20.0, 30.0, 40.0);
        rect.scale(2.0);
        assert_eq!(rect.left, 20.0);
        assert_eq!(rect.top, 40.0);
        assert_eq!(rect.right, 60.0);
        assert_eq!(rect.bottom, 80.0);
    }

    #[test]
    fn test_type_aliases() {
        let p64 = Point64::new(100, 200);
        let pd = PointD::new(10.5, 20.5);
        let r64 = Rect64::new(0, 0, 100, 200);
        let rd = RectD::new(0.0, 0.0, 100.0, 200.0);

        assert_eq!(p64.x, 100);
        assert_eq!(pd.x, 10.5);
        assert_eq!(r64.width(), 100);
        assert_eq!(rd.width(), 100.0);
    }

    #[test]
    fn test_path_types() {
        let mut path64: Path64 = vec![
            Point64::new(0, 0),
            Point64::new(100, 0),
            Point64::new(100, 100),
        ];

        path64.push(Point64::new(0, 100));
        assert_eq!(path64.len(), 4);

        let paths64: Paths64 = vec![path64];
        assert_eq!(paths64.len(), 1);
        assert_eq!(paths64[0].len(), 4);
    }

    #[test]
    fn test_invalid_points() {
        assert_eq!(INVALID_POINT64.x, i64::MAX);
        assert_eq!(INVALID_POINT64.y, i64::MAX);
        assert_eq!(INVALID_POINTD.x, f64::MAX);
        assert_eq!(INVALID_POINTD.y, f64::MAX);
    }

    #[test]
    fn test_mid_point() {
        let p1 = Point64::new(0, 0);
        let p2 = Point64::new(100, 200);
        let mid = mid_point(p1, p2);
        assert_eq!(mid.x, 50);
        assert_eq!(mid.y, 100);

        let p3 = PointD::new(1.0, 3.0);
        let p4 = PointD::new(5.0, 7.0);
        let mid2 = mid_point(p3, p4);
        assert_eq!(mid2.x, 3.0);
        assert_eq!(mid2.y, 5.0);
    }

    #[test]
    fn test_cross_product_three_points() {
        // Test with points that form a right turn (negative cross product)
        let p1 = Point64::new(0, 0);
        let p2 = Point64::new(1, 0);
        let p3 = Point64::new(1, 1);
        let cross = cross_product_three_points(p1, p2, p3);
        assert_eq!(cross, 1.0);

        // Test with points that form a left turn (positive cross product)
        let p4 = Point64::new(0, 0);
        let p5 = Point64::new(1, 0);
        let p6 = Point64::new(1, -1);
        let cross2 = cross_product_three_points(p4, p5, p6);
        assert_eq!(cross2, -1.0);

        // Test collinear points (zero cross product)
        let p7 = Point64::new(0, 0);
        let p8 = Point64::new(1, 1);
        let p9 = Point64::new(2, 2);
        let cross3 = cross_product_three_points(p7, p8, p9);
        assert_eq!(cross3, 0.0);
    }

    #[test]
    fn test_cross_product_two_vectors() {
        let vec1 = Point64::new(1, 0);
        let vec2 = Point64::new(0, 1);
        let cross = cross_product_two_vectors(vec1, vec2);

        // Check actual calculation: vec1.y * vec2.x - vec2.y * vec1.x = 0*0 - 1*1 = -1
        assert_eq!(cross, -1.0);

        let vec3 = Point64::new(2, 3);
        let vec4 = Point64::new(4, 5);
        let cross2 = cross_product_two_vectors(vec3, vec4);
        // vec3.y * vec4.x - vec4.y * vec3.x = 3*4 - 5*2 = 12 - 10 = 2
        assert_eq!(cross2, 2.0);
    }

    #[test]
    fn test_dot_product_three_points() {
        // Test with perpendicular vectors (dot product = 0)
        let p1 = Point64::new(0, 0);
        let p2 = Point64::new(1, 0);
        let p3 = Point64::new(1, 1);
        let dot = dot_product_three_points(p1, p2, p3);
        assert_eq!(dot, 0.0); // (1,0) . (0,1) = 0

        // Test with parallel vectors (positive dot product)
        let p4 = Point64::new(0, 0);
        let p5 = Point64::new(1, 0);
        let p6 = Point64::new(2, 0);
        let dot2 = dot_product_three_points(p4, p5, p6);
        assert_eq!(dot2, 1.0); // (1,0) . (1,0) = 1

        // Test with opposite vectors (negative dot product)
        let p7 = Point64::new(0, 0);
        let p8 = Point64::new(1, 0);
        let p9 = Point64::new(0, 0);
        let dot3 = dot_product_three_points(p7, p8, p9);
        assert_eq!(dot3, -1.0); // (1,0) . (-1,0) = -1
    }

    #[test]
    fn test_dot_product_two_vectors() {
        let vec1 = Point64::new(3, 4);
        let vec2 = Point64::new(2, 1);
        let dot = dot_product_two_vectors(vec1, vec2);
        assert_eq!(dot, 10.0); // 3*2 + 4*1 = 6 + 4 = 10

        // Test with perpendicular vectors
        let vec3 = Point64::new(1, 0);
        let vec4 = Point64::new(0, 1);
        let dot2 = dot_product_two_vectors(vec3, vec4);
        assert_eq!(dot2, 0.0);
    }

    #[test]
    fn test_rect_validity() {
        // Test valid rectangle creation
        let valid_rect = Rect64::new_with_validity(true);
        assert!(valid_rect.is_valid());
        assert_eq!(valid_rect.left, 0);
        assert_eq!(valid_rect.right, 0);

        // Test invalid rectangle creation
        let invalid_rect = Rect64::new_with_validity(false);
        assert!(!invalid_rect.is_valid());

        // Test invalid rectangle factory method
        let invalid_rect2 = Rect64::invalid();
        assert!(!invalid_rect2.is_valid());
    }

    #[test]
    fn test_rect_midpoint() {
        let rect = Rect64::new(10, 20, 30, 40);
        let mid = rect.mid_point();
        assert_eq!(mid.x, 20); // (10 + 30) / 2
        assert_eq!(mid.y, 30); // (20 + 40) / 2
    }

    #[test]
    fn test_rect_as_path() {
        let rect = Rect64::new(0, 0, 100, 200);
        let path = rect.as_path();
        assert_eq!(path.len(), 4);

        // Clockwise from top-left
        assert_eq!(path[0], Point64::new(0, 0)); // top-left
        assert_eq!(path[1], Point64::new(100, 0)); // top-right
        assert_eq!(path[2], Point64::new(100, 200)); // bottom-right
        assert_eq!(path[3], Point64::new(0, 200)); // bottom-left
    }

    #[test]
    fn test_rect_contains_point() {
        let rect = Rect64::new(10, 10, 100, 100);

        // Point inside (exclusive bounds)
        assert!(rect.contains_point(&Point64::new(50, 50)));

        // Points on edges should not be contained (exclusive)
        assert!(!rect.contains_point(&Point64::new(10, 50))); // left edge
        assert!(!rect.contains_point(&Point64::new(100, 50))); // right edge
        assert!(!rect.contains_point(&Point64::new(50, 10))); // top edge
        assert!(!rect.contains_point(&Point64::new(50, 100))); // bottom edge

        // Points outside
        assert!(!rect.contains_point(&Point64::new(5, 50)));
        assert!(!rect.contains_point(&Point64::new(150, 50)));
    }

    #[test]
    fn test_rect_contains_rect() {
        let outer = Rect64::new(0, 0, 100, 100);
        let inner = Rect64::new(10, 10, 90, 90);
        let overlapping = Rect64::new(50, 50, 150, 150);
        let outside = Rect64::new(200, 200, 300, 300);

        assert!(outer.contains_rect(&inner));
        assert!(!outer.contains_rect(&overlapping));
        assert!(!outer.contains_rect(&outside));

        // Same rectangle should contain itself
        assert!(outer.contains_rect(&outer));
    }

    #[test]
    fn test_rect_intersects() {
        let rect1 = Rect64::new(0, 0, 100, 100);
        let rect2 = Rect64::new(50, 50, 150, 150); // overlapping
        let rect3 = Rect64::new(200, 200, 300, 300); // separate
        let rect4 = Rect64::new(100, 0, 200, 100); // touching edge

        assert!(rect1.intersects(&rect2));
        assert!(!rect1.intersects(&rect3));
        assert!(rect1.intersects(&rect4)); // touching edges do intersect

        // Rectangle intersects with itself
        assert!(rect1.intersects(&rect1));
    }

    #[test]
    fn test_rect_equality() {
        let rect1 = Rect64::new(10, 20, 30, 40);
        let rect2 = Rect64::new(10, 20, 30, 40);
        let rect3 = Rect64::new(10, 20, 30, 41);

        assert_eq!(rect1, rect2);
        assert_ne!(rect1, rect3);
    }

    #[test]
    fn test_rect_union_operator() {
        let mut rect1 = Rect64::new(0, 0, 50, 50);
        let rect2 = Rect64::new(25, 25, 100, 100);

        rect1 += rect2;

        // Result should be bounding box of both rectangles
        assert_eq!(rect1.left, 0);
        assert_eq!(rect1.top, 0);
        assert_eq!(rect1.right, 100);
        assert_eq!(rect1.bottom, 100);
    }

    #[test]
    fn test_constants() {
        use constants::*;

        assert!((PI - std::f64::consts::PI).abs() < 0.000_000_1);
        assert_eq!(CLIPPER2_MAX_DEC_PRECISION, 8);
        assert_eq!(MIN_COORD, -MAX_COORD);
        assert_eq!(INVALID, i64::MAX);
        // Constants are verified at compile time - these runtime checks are redundant
    }

    #[test]
    fn test_do_error() {
        use errors::*;

        let result = do_error(PRECISION_ERROR_I);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().description(), PRECISION_ERROR);

        let result2 = do_error(SCALE_ERROR_I);
        assert!(result2.is_err());
        assert_eq!(result2.unwrap_err().description(), SCALE_ERROR);

        let result3 = do_error(999); // unknown error
        assert!(result3.is_err());
        assert_eq!(result3.unwrap_err().description(), "Unknown error");
    }

    #[test]
    fn test_tri_sign() {
        assert_eq!(tri_sign(10), 1);
        assert_eq!(tri_sign(-10), -1);
        assert_eq!(tri_sign(0), 0);
        assert_eq!(tri_sign(i64::MAX), 1);
        assert_eq!(tri_sign(i64::MIN), -1);
        assert_eq!(tri_sign(1), 1);
        assert_eq!(tri_sign(-1), -1);
    }

    #[test]
    fn test_multiply_u64() {
        // Test simple cases
        let result = multiply_u64(0, 0);
        assert_eq!(result.hi, 0);
        assert_eq!(result.lo, 0);

        let result = multiply_u64(1, 1);
        assert_eq!(result.hi, 0);
        assert_eq!(result.lo, 1);

        let result = multiply_u64(10, 20);
        assert_eq!(result.hi, 0);
        assert_eq!(result.lo, 200);

        // Test case that would overflow 64-bit
        let result = multiply_u64(u64::MAX, 2);
        assert_eq!(result.hi, 1);
        assert_eq!(result.lo, u64::MAX - 1);

        // Test maximum values
        let result = multiply_u64(u64::MAX, u64::MAX);
        assert_eq!(result.hi, u64::MAX - 1);
        assert_eq!(result.lo, 1);
    }

    #[test]
    fn test_products_are_equal() {
        // Test basic equality
        assert!(products_are_equal(2, 3, 6, 1));
        assert!(products_are_equal(2, 3, 1, 6));
        assert!(products_are_equal(4, 5, 10, 2));

        // Test basic inequality
        assert!(!products_are_equal(2, 3, 7, 1));
        assert!(!products_are_equal(4, 5, 10, 3));

        // Test with zero values
        assert!(products_are_equal(0, 5, 0, 10));
        assert!(products_are_equal(5, 0, 10, 0));
        assert!(products_are_equal(0, 5, 1, 0)); // Both products are 0
        assert!(!products_are_equal(0, 5, 1, 1)); // 0 != 1

        // Test with negative values
        assert!(products_are_equal(-2, 3, 2, -3));
        assert!(products_are_equal(-2, -3, 2, 3));
        assert!(!products_are_equal(-2, 3, 2, 3));

        // Test large values that might cause overflow
        let large = 1000000000i64;
        assert!(products_are_equal(large, 2, 2 * large, 1));
        assert!(products_are_equal(large, large, large * large, 1));

        // Test edge cases with max values
        assert!(products_are_equal(i64::MAX, 0, 0, i64::MAX));
        assert!(products_are_equal(i64::MIN, 0, 0, i64::MIN));

        // Test sign differentiation - this is important for the algorithm
        assert!(products_are_equal(1, -1, -1, 1)); // Both products are -1
        assert!(products_are_equal(-1, -1, 1, 1)); // Both positive results
        assert!(!products_are_equal(1, -1, 1, 1)); // -1 != 1
    }

    #[test]
    fn test_strip_duplicates_path() {
        // Test open path with duplicates
        let mut open_path = vec![
            Point64::new(0, 0),
            Point64::new(0, 0), // duplicate
            Point64::new(10, 10),
            Point64::new(10, 10), // duplicate
            Point64::new(20, 20),
        ];
        strip_duplicates_path(&mut open_path, false);
        assert_eq!(open_path.len(), 3);
        assert_eq!(open_path[0], Point64::new(0, 0));
        assert_eq!(open_path[1], Point64::new(10, 10));
        assert_eq!(open_path[2], Point64::new(20, 20));

        // Test closed path with duplicates including wrap-around
        let mut closed_path = vec![
            Point64::new(0, 0),
            Point64::new(10, 0),
            Point64::new(10, 10),
            Point64::new(0, 10),
            Point64::new(0, 0), // should be removed for closed path
        ];
        strip_duplicates_path(&mut closed_path, true);
        assert_eq!(closed_path.len(), 4);
        assert_eq!(closed_path[0], Point64::new(0, 0));
        assert_eq!(closed_path[3], Point64::new(0, 10));

        // Test path with no duplicates
        let mut no_dups = vec![
            Point64::new(0, 0),
            Point64::new(10, 10),
            Point64::new(20, 20),
        ];
        let original = no_dups.clone();
        strip_duplicates_path(&mut no_dups, false);
        assert_eq!(no_dups, original);

        // Test empty path
        let mut empty: Path64 = vec![];
        strip_duplicates_path(&mut empty, true);
        assert!(empty.is_empty());

        // Test single point path
        let mut single = vec![Point64::new(0, 0)];
        strip_duplicates_path(&mut single, true);
        assert_eq!(single.len(), 1);
    }

    #[test]
    fn test_strip_duplicates_paths() {
        let mut paths = vec![
            vec![
                Point64::new(0, 0),
                Point64::new(0, 0), // duplicate
                Point64::new(10, 10),
            ],
            vec![
                Point64::new(20, 20),
                Point64::new(30, 30),
                Point64::new(20, 20), // wrap-around duplicate
            ],
        ];

        strip_duplicates_paths(&mut paths, true);

        // First path should have duplicate removed
        assert_eq!(paths[0].len(), 2);
        assert_eq!(paths[0][0], Point64::new(0, 0));
        assert_eq!(paths[0][1], Point64::new(10, 10));

        // Second path should have wrap-around duplicate removed
        assert_eq!(paths[1].len(), 2);
        assert_eq!(paths[1][0], Point64::new(20, 20));
        assert_eq!(paths[1][1], Point64::new(30, 30));
    }

    #[test]
    fn test_check_precision_range() {
        use constants::CLIPPER2_MAX_DEC_PRECISION;
        use errors::PRECISION_ERROR_I;

        // Test valid precision - should not change
        let mut precision = 5;
        let mut error_code = 0;
        check_precision_range(&mut precision, &mut error_code);
        assert_eq!(precision, 5);
        assert_eq!(error_code, 0);

        // Test maximum valid precision
        let mut precision = CLIPPER2_MAX_DEC_PRECISION;
        let mut error_code = 0;
        check_precision_range(&mut precision, &mut error_code);
        assert_eq!(precision, CLIPPER2_MAX_DEC_PRECISION);
        assert_eq!(error_code, 0);

        // Test minimum valid precision
        let mut precision = -CLIPPER2_MAX_DEC_PRECISION;
        let mut error_code = 0;
        check_precision_range(&mut precision, &mut error_code);
        assert_eq!(precision, -CLIPPER2_MAX_DEC_PRECISION);
        assert_eq!(error_code, 0);

        // Test positive overflow - should clamp and set error
        let mut precision = CLIPPER2_MAX_DEC_PRECISION + 1;
        let mut error_code = 0;
        check_precision_range(&mut precision, &mut error_code);
        assert_eq!(precision, CLIPPER2_MAX_DEC_PRECISION);
        assert_eq!(error_code, PRECISION_ERROR_I);

        // Test negative overflow - should clamp and set error
        let mut precision = -CLIPPER2_MAX_DEC_PRECISION - 1;
        let mut error_code = 0;
        check_precision_range(&mut precision, &mut error_code);
        assert_eq!(precision, -CLIPPER2_MAX_DEC_PRECISION);
        assert_eq!(error_code, PRECISION_ERROR_I);

        // Test extreme positive value
        let mut precision = i32::MAX;
        let mut error_code = 0;
        check_precision_range(&mut precision, &mut error_code);
        assert_eq!(precision, CLIPPER2_MAX_DEC_PRECISION);
        assert_eq!(error_code, PRECISION_ERROR_I);

        // Test extreme negative value
        let mut precision = i32::MIN;
        let mut error_code = 0;
        check_precision_range(&mut precision, &mut error_code);
        assert_eq!(precision, -CLIPPER2_MAX_DEC_PRECISION);
        assert_eq!(error_code, PRECISION_ERROR_I);
    }

    #[test]
    fn test_check_precision_range_simple() {
        use constants::CLIPPER2_MAX_DEC_PRECISION;

        // Test convenience function
        let mut precision = CLIPPER2_MAX_DEC_PRECISION + 5;
        check_precision_range_simple(&mut precision);
        assert_eq!(precision, CLIPPER2_MAX_DEC_PRECISION);

        let mut precision = -CLIPPER2_MAX_DEC_PRECISION - 3;
        check_precision_range_simple(&mut precision);
        assert_eq!(precision, -CLIPPER2_MAX_DEC_PRECISION);

        let mut precision = 3;
        check_precision_range_simple(&mut precision);
        assert_eq!(precision, 3); // Should remain unchanged
    }

    #[test]
    fn test_get_bounds_path() {
        // Test basic rectangular path
        let path: Path64 = vec![
            Point64::new(10, 20),
            Point64::new(100, 30),
            Point64::new(50, 80),
            Point64::new(0, 10),
        ];

        let bounds = get_bounds_path(&path);
        assert_eq!(bounds.left, 0);
        assert_eq!(bounds.top, 10);
        assert_eq!(bounds.right, 100);
        assert_eq!(bounds.bottom, 80);

        // Test single point path
        let single_path: Path64 = vec![Point64::new(42, 37)];
        let single_bounds = get_bounds_path(&single_path);
        assert_eq!(single_bounds.left, 42);
        assert_eq!(single_bounds.top, 37);
        assert_eq!(single_bounds.right, 42);
        assert_eq!(single_bounds.bottom, 37);

        // Test empty path - should return invalid bounds
        let empty_path: Path64 = vec![];
        let empty_bounds = get_bounds_path(&empty_path);
        assert_eq!(empty_bounds.left, i64::MAX);
        assert_eq!(empty_bounds.top, i64::MAX);
        assert_eq!(empty_bounds.right, i64::MIN);
        assert_eq!(empty_bounds.bottom, i64::MIN);
    }

    #[test]
    fn test_get_bounds_path_double() {
        // Test with floating-point path
        let path: PathD = vec![
            PointD::new(10.5, 20.7),
            PointD::new(100.3, 30.1),
            PointD::new(50.9, 80.4),
            PointD::new(0.2, 10.8),
        ];

        let bounds = get_bounds_path(&path);
        assert_eq!(bounds.left, 0.2);
        assert_eq!(bounds.top, 10.8);
        assert_eq!(bounds.right, 100.3);
        assert_eq!(bounds.bottom, 80.4);
    }

    #[test]
    fn test_get_bounds_paths() {
        // Test multiple paths
        let paths: Paths64 = vec![
            vec![Point64::new(0, 0), Point64::new(50, 25)],
            vec![Point64::new(25, 50), Point64::new(100, 75)],
            vec![Point64::new(-10, -5), Point64::new(30, 40)],
        ];

        let bounds = get_bounds_paths(&paths);
        assert_eq!(bounds.left, -10);
        assert_eq!(bounds.top, -5);
        assert_eq!(bounds.right, 100);
        assert_eq!(bounds.bottom, 75);

        // Test empty paths
        let empty_paths: Paths64 = vec![];
        let empty_bounds = get_bounds_paths(&empty_paths);
        assert_eq!(empty_bounds.left, i64::MAX);
        assert_eq!(empty_bounds.right, i64::MIN);

        // Test paths with empty paths inside
        let mixed_paths: Paths64 = vec![
            vec![Point64::new(10, 20)],
            vec![], // empty path
            vec![Point64::new(30, 40)],
        ];
        let mixed_bounds = get_bounds_paths(&mixed_paths);
        assert_eq!(mixed_bounds.left, 10);
        assert_eq!(mixed_bounds.top, 20);
        assert_eq!(mixed_bounds.right, 30);
        assert_eq!(mixed_bounds.bottom, 40);
    }

    #[test]
    fn test_get_bounds_path_convert() {
        // Test converting from i32 path to i64 bounds
        let path32: Path<i32> = vec![
            Point::new(10i32, 20i32),
            Point::new(100i32, 30i32),
            Point::new(50i32, 80i32),
        ];

        let bounds64: Rect64 = get_bounds_path_convert(&path32);
        assert_eq!(bounds64.left, 10i64);
        assert_eq!(bounds64.top, 20i64);
        assert_eq!(bounds64.right, 100i64);
        assert_eq!(bounds64.bottom, 80i64);

        // Test converting from f32 path to f64 bounds
        let pathf32: Path<f32> = vec![Point::new(10.5f32, 20.7f32), Point::new(100.3f32, 30.1f32)];

        let boundsf64: RectD = get_bounds_path_convert(&pathf32);
        // Use a more generous epsilon for f32 to f64 conversion
        const TOLERANCE: f64 = 1e-6;
        assert!((boundsf64.left - 10.5).abs() < TOLERANCE);
        assert!((boundsf64.top - 20.700000762939453).abs() < TOLERANCE); // f32 precision loss
        assert!((boundsf64.right - 100.30000305175781).abs() < TOLERANCE);
        assert!((boundsf64.bottom - 30.100000381469727).abs() < TOLERANCE);
    }

    #[test]
    fn test_get_bounds_paths_convert() {
        // Test converting multiple paths
        let paths32: Paths<i32> = vec![
            vec![Point::new(10i32, 20i32), Point::new(50i32, 25i32)],
            vec![Point::new(25i32, 50i32), Point::new(100i32, 75i32)],
        ];

        let bounds64: Rect64 = get_bounds_paths_convert(&paths32);
        assert_eq!(bounds64.left, 10i64);
        assert_eq!(bounds64.top, 20i64);
        assert_eq!(bounds64.right, 100i64);
        assert_eq!(bounds64.bottom, 75i64);
    }

    #[test]
    fn test_get_bounds_extreme_values() {
        // Test with extreme coordinate values
        let extreme_path: Path64 = vec![
            Point64::new(i64::MIN + 100, i64::MIN + 200),
            Point64::new(i64::MAX - 100, i64::MAX - 200),
            Point64::new(0, 0),
        ];

        let bounds = get_bounds_path(&extreme_path);
        assert_eq!(bounds.left, i64::MIN + 100);
        assert_eq!(bounds.top, i64::MIN + 200);
        assert_eq!(bounds.right, i64::MAX - 100);
        assert_eq!(bounds.bottom, i64::MAX - 200);
    }

    #[test]
    fn test_get_bounds_negative_coordinates() {
        // Test with all negative coordinates
        let negative_path: Path64 = vec![
            Point64::new(-100, -200),
            Point64::new(-50, -150),
            Point64::new(-75, -175),
        ];

        let bounds = get_bounds_path(&negative_path);
        assert_eq!(bounds.left, -100);
        assert_eq!(bounds.top, -200);
        assert_eq!(bounds.right, -50);
        assert_eq!(bounds.bottom, -150);
    }

    #[test]
    fn test_get_bounds_identical_points() {
        // Test with identical points (degenerate case)
        let identical_path: Path64 = vec![
            Point64::new(42, 37),
            Point64::new(42, 37),
            Point64::new(42, 37),
        ];

        let bounds = get_bounds_path(&identical_path);
        assert_eq!(bounds.left, 42);
        assert_eq!(bounds.top, 37);
        assert_eq!(bounds.right, 42);
        assert_eq!(bounds.bottom, 37);

        // Verify it results in a valid zero-area rectangle
        assert_eq!(bounds.width(), 0);
        assert_eq!(bounds.height(), 0);
        assert!(bounds.is_empty()); // Zero-size rectangles are empty when left==right or top==bottom
    }
}
