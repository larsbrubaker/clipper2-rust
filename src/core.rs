//! Core types and structures for Clipper2
//! 
//! Direct port from clipper.core.h
//! This module contains the fundamental data types and basic operations

use num_traits::{Num, Zero, Float};
use std::fmt::{Debug, Display};

/// Fill rule determines how polygons with self-intersections are filled
/// Direct port from clipper.core.h line 108
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum FillRule {
    /// Even-odd fill rule (also known as Alternate)
    EvenOdd,
    /// Non-zero fill rule (also known as Winding) 
    NonZero,
    /// Positive fill rule
    Positive,
    /// Negative fill rule
    Negative,
}

impl Default for FillRule {
    fn default() -> Self {
        FillRule::EvenOdd
    }
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
    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    
    /// Subtract two points
    pub fn sub(self, other: Self) -> Self {
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
        self.add(rhs)
    }
}

impl<T> std::ops::Sub for Point<T>
where
    T: Num + Copy,
{
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
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
#[derive(Debug, Clone, Copy, PartialEq, Default)]
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
        Self { left, top, right, bottom }
    }
    
    /// Check if rectangle is valid (left <= right, top <= bottom)
    pub fn is_valid(&self) -> bool {
        self.left <= self.right && self.top <= self.bottom
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fill_rule_default() {
        assert_eq!(FillRule::default(), FillRule::EvenOdd);
    }
    
    #[test] 
    fn test_fill_rule_variants() {
        let rules = [FillRule::EvenOdd, FillRule::NonZero, FillRule::Positive, FillRule::Negative];
        assert_eq!(rules.len(), 4);
        
        // Test each variant is unique
        for i in 0..rules.len() {
            for j in (i+1)..rules.len() {
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
        assert!(!empty_rect.is_valid());
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
}