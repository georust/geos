use libc::{c_int, size_t};

use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum CoordDimensions {
    OneD,
    TwoD,
    ThreeD,
}

impl TryFrom<u32> for CoordDimensions {
    type Error = &'static str;

    fn try_from(dimensions: u32) -> Result<Self, Self::Error> {
        match dimensions {
            1 => Ok(CoordDimensions::OneD),
            2 => Ok(CoordDimensions::TwoD),
            3 => Ok(CoordDimensions::ThreeD),
            _ => Err("dimensions must be >= 1 and <= 3"),
        }
    }
}

#[allow(clippy::from_over_into)]
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

impl TryFrom<c_int> for Dimensions {
    type Error = &'static str;

    fn try_from(dimensions: c_int) -> Result<Self, Self::Error> {
        match dimensions {
            2 => Ok(Dimensions::TwoD),
            3 => Ok(Dimensions::ThreeD),
            x if x > 3 => Ok(Dimensions::Other(x as _)),
            _ => Err("dimensions must be > 1"),
        }
    }
}

#[allow(clippy::from_over_into)]
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
pub enum OutputDimension {
    TwoD,
    ThreeD,
}

impl TryFrom<c_int> for OutputDimension {
    type Error = &'static str;

    fn try_from(dimensions: c_int) -> Result<Self, Self::Error> {
        match dimensions {
            2 => Ok(OutputDimension::TwoD),
            3 => Ok(OutputDimension::ThreeD),
            _ => Err("dimension must be 2 or 3"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for OutputDimension {
    fn into(self) -> c_int {
        match self {
            OutputDimension::TwoD => 2,
            OutputDimension::ThreeD => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

impl TryFrom<c_int> for ByteOrder {
    type Error = &'static str;

    fn try_from(order: c_int) -> Result<Self, Self::Error> {
        match order {
            0 => Ok(ByteOrder::BigEndian),
            _ => Ok(ByteOrder::LittleEndian),
        }
    }
}

#[allow(clippy::from_over_into)]
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
pub enum GeometryTypes {
    Point,
    LineString,
    LinearRing,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
    #[doc(hidden)]
    __Unknown(u32),
}

impl TryFrom<c_int> for GeometryTypes {
    type Error = &'static str;

    fn try_from(dimensions: c_int) -> Result<Self, Self::Error> {
        match dimensions {
            0 => Ok(GeometryTypes::Point),
            1 => Ok(GeometryTypes::LineString),
            2 => Ok(GeometryTypes::LinearRing),
            3 => Ok(GeometryTypes::Polygon),
            4 => Ok(GeometryTypes::MultiPoint),
            5 => Ok(GeometryTypes::MultiLineString),
            6 => Ok(GeometryTypes::MultiPolygon),
            7 => Ok(GeometryTypes::GeometryCollection),
            x => Ok(GeometryTypes::__Unknown(x as _)),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for GeometryTypes {
    fn into(self) -> c_int {
        match self {
            GeometryTypes::Point => 0,
            GeometryTypes::LineString => 1,
            GeometryTypes::LinearRing => 2,
            GeometryTypes::Polygon => 3,
            GeometryTypes::MultiPoint => 4,
            GeometryTypes::MultiLineString => 5,
            GeometryTypes::MultiPolygon => 6,
            GeometryTypes::GeometryCollection => 7,
            GeometryTypes::__Unknown(x) => x as _,
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

#[allow(clippy::from_over_into)]
impl Into<c_int> for Orientation {
    fn into(self) -> c_int {
        match self {
            Orientation::CounterClockwise => -1,
            Orientation::Clockwise => 0,
            Orientation::Colinear => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Ordinate {
    X,
    Y,
    Z,
}

impl TryFrom<size_t> for Ordinate {
    type Error = &'static str;

    fn try_from(ordinate: size_t) -> Result<Self, Self::Error> {
        match ordinate {
            0 => Ok(Ordinate::X),
            1 => Ok(Ordinate::Y),
            2 => Ok(Ordinate::Z),
            _ => Err("ordinate value must be >= 0 and <= 2"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<u32> for Ordinate {
    fn into(self) -> u32 {
        match self {
            Ordinate::X => 0,
            Ordinate::Y => 1,
            Ordinate::Z => 2,
        }
    }
}

#[cfg(any(feature = "v3_6_0", feature = "dox"))]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Precision {
    ValidOutput,
    NoTopo,
    KeepCollapsed,
}

#[cfg(any(feature = "v3_6_0", feature = "dox"))]
impl TryFrom<c_int> for Precision {
    type Error = &'static str;

    fn try_from(order: c_int) -> Result<Self, Self::Error> {
        match order {
            0 => Ok(Precision::ValidOutput),
            1 => Ok(Precision::NoTopo),
            2 => Ok(Precision::KeepCollapsed),
            _ => Err("Unknown precision type"),
        }
    }
}

#[cfg(any(feature = "v3_6_0", feature = "dox"))]
#[allow(clippy::from_over_into)]
impl Into<c_int> for Precision {
    fn into(self) -> c_int {
        match self {
            Precision::ValidOutput => 0,
            Precision::NoTopo => 1,
            Precision::KeepCollapsed => 2,
        }
    }
}

/// Join styles for a [`Geometry`](crate::Geometry) [buffer](crate::Geom::buffer_with_style) operation
#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum JoinStyle {
    /// Specifies a round join style.
    #[default]
    Round,
    /// Specifies a mitre join style.
    Mitre,
    /// Specifies a bevel join style.
    Bevel,
}

impl TryFrom<c_int> for JoinStyle {
    type Error = &'static str;

    fn try_from(join_style: c_int) -> Result<Self, Self::Error> {
        match join_style {
            1 => Ok(JoinStyle::Round),
            2 => Ok(JoinStyle::Mitre),
            3 => Ok(JoinStyle::Bevel),
            _ => Err("Unknown join style"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for JoinStyle {
    fn into(self) -> c_int {
        match self {
            JoinStyle::Round => 1,
            JoinStyle::Mitre => 2,
            JoinStyle::Bevel => 3,
        }
    }
}

/// End cap styles for a [`Geometry`](crate::Geometry) [buffer](crate::Geom::buffer_with_style) operation
#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum CapStyle {
    /// Specifies a round line buffer end cap style.
    #[default]
    Round,
    /// Specifies a flat line buffer end cap style.
    Flat,
    /// Specifies a square line buffer end cap style.
    Square,
}

impl TryFrom<c_int> for CapStyle {
    type Error = &'static str;

    fn try_from(cap_style: c_int) -> Result<Self, Self::Error> {
        match cap_style {
            1 => Ok(CapStyle::Round),
            2 => Ok(CapStyle::Flat),
            3 => Ok(CapStyle::Square),
            _ => Err("Unknown cap style"),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for CapStyle {
    fn into(self) -> c_int {
        match self {
            CapStyle::Round => 1,
            CapStyle::Flat => 2,
            CapStyle::Square => 3,
        }
    }
}
