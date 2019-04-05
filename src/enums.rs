use libc::c_int;

// use std::convert::TryFrom;
// TODO: remove this implementation when 1.34 is released.
pub trait TryFrom<T>: Sized {
    type Error;
    fn try_from(value: T) -> Result<Self, Self::Error>;
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum CoordDimensions {
    OneD,
    TwoD,
    ThreeD,
}

impl From<u32> for CoordDimensions {
    fn from(dimensions: u32) -> Self {
        match dimensions {
            1 => CoordDimensions::OneD,
            2 => CoordDimensions::TwoD,
            3 => CoordDimensions::ThreeD,
            _ => panic!("dimensions must be >= 1 and <= 3"),
        }
    }
}

impl Into<u32> for CoordDimensions {
    fn into(self) -> u32 {
        match self {
            CoordDimensions::OneD => 1,
            CoordDimensions::TwoD => 2,
            CoordDimensions::ThreeD => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Dimensions {
    TwoD,
    ThreeD,
    Other(u32),
}

impl From<c_int> for Dimensions {
    fn from(dimensions: c_int) -> Self {
        match dimensions {
            2 => Dimensions::TwoD,
            3 => Dimensions::ThreeD,
            x if x > 3 => Dimensions::Other(x as _),
            _ => panic!("dimensions must be > 1"),
        }
    }
}

impl Into<c_int> for Dimensions {
    fn into(self) -> c_int {
        match self {
            Dimensions::TwoD => 2,
            Dimensions::ThreeD => 3,
            Dimensions::Other(dim) => dim as _,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

impl From<c_int> for ByteOrder {
    fn from(dimensions: c_int) -> Self {
        match dimensions {
            0 => ByteOrder::BigEndian,
            _ => ByteOrder::LittleEndian,
        }
    }
}

impl Into<c_int> for ByteOrder {
    fn into(self) -> c_int {
        match self {
            ByteOrder::BigEndian => 0,
            ByteOrder::LittleEndian => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
#[repr(C)]
pub enum GGeomTypes {
    Point,
    LineString,
    LinearRing,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
    #[doc(hidden)]
    __Unknonwn(u32),
}

impl From<c_int> for GGeomTypes {
    fn from(dimensions: c_int) -> Self {
        match dimensions {
            0 => GGeomTypes::Point,
            1 => GGeomTypes::LineString,
            2 => GGeomTypes::LinearRing,
            3 => GGeomTypes::Polygon,
            4 => GGeomTypes::MultiPoint,
            5 => GGeomTypes::MultiLineString,
            6 => GGeomTypes::MultiPolygon,
            7 => GGeomTypes::GeometryCollection,
            x => GGeomTypes::__Unknonwn(x as _),
        }
    }
}

impl Into<c_int> for GGeomTypes {
    fn into(self) -> c_int {
        match self {
            GGeomTypes::Point => 0,
            GGeomTypes::LineString => 1,
            GGeomTypes::LinearRing => 2,
            GGeomTypes::Polygon => 3,
            GGeomTypes::MultiPoint => 4,
            GGeomTypes::MultiLineString => 5,
            GGeomTypes::MultiPolygon => 6,
            GGeomTypes::GeometryCollection => 7,
            GGeomTypes::__Unknonwn(x) => x as _,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Orientation {
    /// If reaching P takes a counter-clockwise (left) turn.
    CounterClockwise,
    /// If reaching P takes a clockwise (right) turn
    Clockwise,
    /// if P is collinear with A-B.
    Colinear,
}

impl From<c_int> for Orientation {
    fn from(orientation: c_int) -> Self {
        match orientation {
            -1 => Orientation::CounterClockwise,
             0 => Orientation::Clockwise,
             1 => Orientation::Colinear,
             _ => panic!("invalid value for Orientation!"),
        }
    }
}

impl TryFrom<c_int> for Orientation {
    type Error = &'static str;

    fn try_from(orientation: c_int) -> Result<Self, Self::Error> {
        match orientation {
            -1 => Ok(Orientation::CounterClockwise),
             0 => Ok(Orientation::Clockwise),
             1 => Ok(Orientation::Colinear),
             _ => Err("value must be -1, 0 or 1"),
        }
    }
}

impl Into<c_int> for Orientation {
    fn into(self) -> c_int {
        match self {
            Orientation::CounterClockwise => -1,
            Orientation::Clockwise => 0,
            Orientation::Colinear => 1,
        }
    }
}
