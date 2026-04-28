//! LSL data types and constants

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// LSL value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LSLValue {
    Integer(i32),
    Float(f32),
    String(String),
    Key(Uuid),
    Vector(LSLVector),
    Rotation(LSLRotation),
    List(Vec<LSLValue>),
}

/// LSL Vector type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LSLVector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// LSL Rotation type (quaternion)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct LSLRotation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub s: f32, // w component
}

impl LSLVector {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self::new(self.x / mag, self.y / mag, self.z / mag)
        } else {
            Self::zero()
        }
    }

    pub fn dot(&self, other: &LSLVector) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &LSLVector) -> LSLVector {
        LSLVector::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn distance(&self, other: &LSLVector) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl std::ops::Add for LSLVector {
    type Output = LSLVector;

    fn add(self, other: LSLVector) -> LSLVector {
        LSLVector::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for LSLVector {
    type Output = LSLVector;

    fn sub(self, other: LSLVector) -> LSLVector {
        LSLVector::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Mul<f32> for LSLVector {
    type Output = LSLVector;

    fn mul(self, scalar: f32) -> LSLVector {
        LSLVector::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl std::ops::Div<f32> for LSLVector {
    type Output = LSLVector;

    fn div(self, scalar: f32) -> LSLVector {
        LSLVector::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl fmt::Display for LSLVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}, {}>", self.x, self.y, self.z)
    }
}

impl LSLRotation {
    pub fn new(x: f32, y: f32, z: f32, s: f32) -> Self {
        Self { x, y, z, s }
    }

    pub fn identity() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }

    pub fn from_euler(roll: f32, pitch: f32, yaw: f32) -> Self {
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();

        Self::new(
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
            cr * cp * cy + sr * sp * sy,
        )
    }

    pub fn normalize(&self) -> Self {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z + self.s * self.s).sqrt();
        if mag > 0.0 {
            Self::new(self.x / mag, self.y / mag, self.z / mag, self.s / mag)
        } else {
            Self::identity()
        }
    }

    pub fn to_euler(&self) -> (f32, f32, f32) {
        let test = self.x * self.y + self.z * self.s;

        if test > 0.499 {
            // Singularity at north pole
            let yaw = 2.0 * self.x.atan2(self.s);
            let pitch = std::f32::consts::PI / 2.0;
            let roll = 0.0;
            return (roll, pitch, yaw);
        }

        if test < -0.499 {
            // Singularity at south pole
            let yaw = -2.0 * self.x.atan2(self.s);
            let pitch = -std::f32::consts::PI / 2.0;
            let roll = 0.0;
            return (roll, pitch, yaw);
        }

        let sqx = self.x * self.x;
        let sqy = self.y * self.y;
        let sqz = self.z * self.z;

        let yaw =
            (2.0 * self.y * self.s - 2.0 * self.x * self.z).atan2(1.0 - 2.0 * sqy - 2.0 * sqz);
        let pitch = (2.0 * test).asin();
        let roll =
            (2.0 * self.x * self.s - 2.0 * self.y * self.z).atan2(1.0 - 2.0 * sqx - 2.0 * sqz);

        (roll, pitch, yaw)
    }
}

impl std::ops::Mul for LSLRotation {
    type Output = LSLRotation;

    fn mul(self, other: LSLRotation) -> LSLRotation {
        LSLRotation::new(
            self.s * other.x + self.x * other.s + self.y * other.z - self.z * other.y,
            self.s * other.y - self.x * other.z + self.y * other.s + self.z * other.x,
            self.s * other.z + self.x * other.y - self.y * other.x + self.z * other.s,
            self.s * other.s - self.x * other.x - self.y * other.y - self.z * other.z,
        )
    }
}

impl fmt::Display for LSLRotation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}, {}, {}, {}>", self.x, self.y, self.z, self.s)
    }
}

impl LSLValue {
    pub fn to_integer(&self) -> i32 {
        match self {
            LSLValue::Integer(i) => *i,
            LSLValue::Float(f) => *f as i32,
            LSLValue::String(s) => parse_leading_integer(s),
            LSLValue::Key(_) => 0,
            LSLValue::Vector(v) => v.magnitude() as i32,
            LSLValue::Rotation(_) => 0,
            LSLValue::List(l) => l.len() as i32,
        }
    }

    /// Convert to float
    pub fn to_float(&self) -> f32 {
        match self {
            LSLValue::Integer(i) => *i as f32,
            LSLValue::Float(f) => *f,
            LSLValue::String(s) => s.parse().unwrap_or(0.0),
            LSLValue::Key(_) => 0.0,
            LSLValue::Vector(v) => v.magnitude(),
            LSLValue::Rotation(_) => 0.0,
            LSLValue::List(l) => l.len() as f32,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            LSLValue::Integer(i) => i.to_string(),
            LSLValue::Float(f) => format!("{:.6}", f),
            LSLValue::String(s) => s.clone(),
            LSLValue::Key(k) => k.to_string(),
            LSLValue::Vector(v) => format!("<{:.6}, {:.6}, {:.6}>", v.x, v.y, v.z),
            LSLValue::Rotation(r) => format!("<{:.6}, {:.6}, {:.6}, {:.6}>", r.x, r.y, r.z, r.s),
            LSLValue::List(l) => {
                let strings: Vec<String> = l.iter().map(|v| v.to_string()).collect();
                strings.join(", ")
            }
        }
    }

    /// Convert to key (UUID)
    pub fn to_key(&self) -> Uuid {
        match self {
            LSLValue::Key(k) => *k,
            LSLValue::String(s) => Uuid::parse_str(s).unwrap_or_else(|_| Uuid::nil()),
            _ => Uuid::nil(),
        }
    }

    /// Convert to vector
    pub fn to_vector(&self) -> LSLVector {
        match self {
            LSLValue::Vector(v) => *v,
            LSLValue::String(s) => parse_vector_string(s).unwrap_or_else(|| LSLVector::zero()),
            _ => LSLVector::zero(),
        }
    }

    /// Convert to rotation
    pub fn to_rotation(&self) -> LSLRotation {
        match self {
            LSLValue::Rotation(r) => *r,
            LSLValue::String(s) => {
                parse_rotation_string(s).unwrap_or_else(|| LSLRotation::identity())
            }
            _ => LSLRotation::identity(),
        }
    }

    /// Convert to list
    pub fn to_list(&self) -> Vec<LSLValue> {
        match self {
            LSLValue::List(l) => l.clone(),
            other => vec![other.clone()],
        }
    }

    /// Check if value is "true" in LSL context
    pub fn is_true(&self) -> bool {
        match self {
            LSLValue::Integer(i) => *i != 0,
            LSLValue::Float(f) => *f != 0.0,
            LSLValue::String(s) => !s.is_empty(),
            LSLValue::Key(k) => *k != Uuid::nil(),
            LSLValue::Vector(v) => v.magnitude() > 0.0,
            LSLValue::Rotation(_) => true, // Rotations are always "true"
            LSLValue::List(l) => !l.is_empty(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            LSLValue::Integer(_) => "integer",
            LSLValue::Float(_) => "float",
            LSLValue::String(_) => "string",
            LSLValue::Key(_) => "key",
            LSLValue::Vector(_) => "vector",
            LSLValue::Rotation(_) => "rotation",
            LSLValue::List(_) => "list",
        }
    }

    pub fn heap_size(&self) -> usize {
        match self {
            LSLValue::Integer(_) => 4,
            LSLValue::Float(_) => 4,
            LSLValue::String(s) => 4 + s.len(),
            LSLValue::Key(_) => 16,
            LSLValue::Vector(_) => 12,
            LSLValue::Rotation(_) => 16,
            LSLValue::List(l) => 4 + l.iter().map(|v| v.heap_size()).sum::<usize>(),
        }
    }

    pub fn type_default(type_name: &str) -> LSLValue {
        match type_name {
            "integer" => LSLValue::Integer(0),
            "float" => LSLValue::Float(0.0),
            "string" => LSLValue::String(String::new()),
            "key" => LSLValue::Key(Uuid::nil()),
            "vector" => LSLValue::Vector(LSLVector::zero()),
            "rotation" => LSLValue::Rotation(LSLRotation::identity()),
            "list" => LSLValue::List(Vec::new()),
            _ => LSLValue::Integer(0),
        }
    }

    pub fn coerce(&self, to_type: &str) -> LSLValue {
        match to_type {
            "integer" => LSLValue::Integer(self.to_integer()),
            "float" => LSLValue::Float(self.to_float()),
            "string" => match self {
                LSLValue::Float(f) => LSLValue::String(format!("{:.6}", f)),
                LSLValue::Vector(v) => {
                    LSLValue::String(format!("<{:.6}, {:.6}, {:.6}>", v.x, v.y, v.z))
                }
                LSLValue::Rotation(r) => {
                    LSLValue::String(format!("<{:.6}, {:.6}, {:.6}, {:.6}>", r.x, r.y, r.z, r.s))
                }
                _ => LSLValue::String(self.to_string()),
            },
            "key" => LSLValue::Key(self.to_key()),
            "vector" => LSLValue::Vector(self.to_vector()),
            "rotation" => LSLValue::Rotation(self.to_rotation()),
            "list" => LSLValue::List(self.to_list()),
            _ => self.clone(),
        }
    }
}

impl fmt::Display for LSLValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

fn parse_leading_integer(s: &str) -> i32 {
    let s = s.trim();
    if s.is_empty() {
        return 0;
    }
    if s.starts_with("0x") || s.starts_with("0X") {
        return i32::from_str_radix(s[2..].trim_end(), 16).unwrap_or(0);
    }
    let mut end = 0;
    let bytes = s.as_bytes();
    if end < bytes.len() && (bytes[end] == b'-' || bytes[end] == b'+') {
        end += 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == 0 || (end == 1 && (bytes[0] == b'-' || bytes[0] == b'+')) {
        return 0;
    }
    s[..end].parse().unwrap_or(0)
}

fn parse_vector_string(s: &str) -> Option<LSLVector> {
    let s = s.trim();
    if !s.starts_with('<') || !s.ends_with('>') {
        return None;
    }

    let inner = &s[1..s.len() - 1];
    let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

    if parts.len() != 3 {
        return None;
    }

    let x = parts[0].parse().ok()?;
    let y = parts[1].parse().ok()?;
    let z = parts[2].parse().ok()?;

    Some(LSLVector::new(x, y, z))
}

/// Parse a rotation from a string like "<0.0, 0.0, 0.0, 1.0>"
fn parse_rotation_string(s: &str) -> Option<LSLRotation> {
    let s = s.trim();
    if !s.starts_with('<') || !s.ends_with('>') {
        return None;
    }

    let inner = &s[1..s.len() - 1];
    let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();

    if parts.len() != 4 {
        return None;
    }

    let x = parts[0].parse().ok()?;
    let y = parts[1].parse().ok()?;
    let z = parts[2].parse().ok()?;
    let s = parts[3].parse().ok()?;

    Some(LSLRotation::new(x, y, z, s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsl_vector_operations() {
        let v1 = LSLVector::new(1.0, 2.0, 3.0);
        let v2 = LSLVector::new(4.0, 5.0, 6.0);

        // Test addition
        let sum = v1 + v2;
        assert_eq!(sum, LSLVector::new(5.0, 7.0, 9.0));

        // Test subtraction
        let diff = v2 - v1;
        assert_eq!(diff, LSLVector::new(3.0, 3.0, 3.0));

        // Test magnitude
        let v3 = LSLVector::new(3.0, 4.0, 0.0);
        assert_eq!(v3.magnitude(), 5.0);

        // Test normalize
        let normalized = v3.normalize();
        assert_eq!(normalized, LSLVector::new(0.6, 0.8, 0.0));

        // Test distance
        let v4 = LSLVector::new(0.0, 0.0, 0.0);
        assert_eq!(v3.distance(&v4), 5.0);
    }

    #[test]
    fn test_lsl_rotation_operations() {
        let r1 = LSLRotation::identity();
        assert_eq!(r1, LSLRotation::new(0.0, 0.0, 0.0, 1.0));

        let r2 = LSLRotation::from_euler(0.0, 0.0, std::f32::consts::PI / 2.0);
        let normalized = r2.normalize();

        // Check that normalization produces a unit quaternion
        let mag = (normalized.x * normalized.x
            + normalized.y * normalized.y
            + normalized.z * normalized.z
            + normalized.s * normalized.s)
            .sqrt();
        assert!((mag - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_lsl_value_conversions() {
        let int_val = LSLValue::Integer(42);
        assert_eq!(int_val.to_float(), 42.0);
        assert_eq!(int_val.to_string(), "42");

        let float_val = LSLValue::Float(3.14);
        assert_eq!(float_val.to_integer(), 3);
        assert_eq!(float_val.to_string(), "3.140000");

        let string_val = LSLValue::String("123".to_string());
        assert_eq!(string_val.to_integer(), 123);
        assert_eq!(string_val.to_float(), 123.0);

        let string_leading = LSLValue::String("42abc".to_string());
        assert_eq!(string_leading.to_integer(), 42);

        let string_hex = LSLValue::String("0xFF".to_string());
        assert_eq!(string_hex.to_integer(), 255);

        let vector_val = LSLValue::Vector(LSLVector::new(3.0, 4.0, 0.0));
        assert_eq!(vector_val.to_float(), 5.0);

        assert!(LSLValue::Integer(1).is_true());
        assert!(!LSLValue::Integer(0).is_true());
        assert!(LSLValue::String("hello".to_string()).is_true());
        assert!(!LSLValue::String("".to_string()).is_true());
    }

    #[test]
    fn test_heap_size() {
        assert_eq!(LSLValue::Integer(0).heap_size(), 4);
        assert_eq!(LSLValue::Float(0.0).heap_size(), 4);
        assert_eq!(LSLValue::String("hello".to_string()).heap_size(), 9);
        assert_eq!(LSLValue::Vector(LSLVector::zero()).heap_size(), 12);
        assert_eq!(LSLValue::Rotation(LSLRotation::identity()).heap_size(), 16);
        let list = LSLValue::List(vec![LSLValue::Integer(1), LSLValue::Float(2.0)]);
        assert_eq!(list.heap_size(), 4 + 4 + 4);
    }

    #[test]
    fn test_type_default() {
        assert_eq!(LSLValue::type_default("integer"), LSLValue::Integer(0));
        assert_eq!(LSLValue::type_default("float"), LSLValue::Float(0.0));
        assert_eq!(
            LSLValue::type_default("string"),
            LSLValue::String(String::new())
        );
        assert_eq!(
            LSLValue::type_default("vector"),
            LSLValue::Vector(LSLVector::zero())
        );
    }

    #[test]
    fn test_coerce() {
        let f = LSLValue::Float(1.5);
        assert_eq!(f.coerce("string"), LSLValue::String("1.500000".to_string()));
        assert_eq!(f.coerce("integer"), LSLValue::Integer(1));

        let i = LSLValue::Integer(42);
        assert_eq!(i.coerce("float"), LSLValue::Float(42.0));
        assert_eq!(i.coerce("string"), LSLValue::String("42".to_string()));
    }

    #[test]
    fn test_vector_string_parsing() {
        let vector_str = "<1.5, -2.0, 3.14>";
        let parsed = parse_vector_string(vector_str).unwrap();
        assert_eq!(parsed, LSLVector::new(1.5, -2.0, 3.14));

        // Test invalid format
        assert!(parse_vector_string("invalid").is_none());
        assert!(parse_vector_string("<1, 2>").is_none()); // Wrong number of components
    }

    #[test]
    fn test_rotation_string_parsing() {
        let rotation_str = "<0.0, 0.0, 0.707, 0.707>";
        let parsed = parse_rotation_string(rotation_str).unwrap();
        assert_eq!(parsed, LSLRotation::new(0.0, 0.0, 0.707, 0.707));

        // Test invalid format
        assert!(parse_rotation_string("invalid").is_none());
        assert!(parse_rotation_string("<1, 2, 3>").is_none()); // Wrong number of components
    }
}
