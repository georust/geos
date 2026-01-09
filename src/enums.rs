use libc::{c_int, size_t};

use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum CoordDimensions {
    #[cfg(not(feature = "v3_12_0"))]
    OneD,
    TwoD,
    ThreeD,
    #[cfg(feature = "v3_12_0")]
    FourD,
}

impl TryFrom<u32> for CoordDimensions {
    type Error = crate::error::Error;

    fn try_from(dimensions: u32) -> Result<Self, Self::Error> {
        match dimensions {
            #[cfg(not(feature = "v3_12_0"))]
            1 => Ok(CoordDimensions::OneD),
            2 => Ok(CoordDimensions::TwoD),
            3 => Ok(CoordDimensions::ThreeD),
            #[cfg(feature = "v3_12_0")]
            4 => Ok(CoordDimensions::FourD),
            #[cfg(not(feature = "v3_12_0"))]
            _ => Err(Self::Error::GenericError(
                "dimensions must be >=1 and <= 3".into(),
            )),
            #[cfg(feature = "v3_12_0")]
            _ => Err(Self::Error::GenericError(
                "dimensions must be >=2 and <= 4".into(),
            )),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<u32> for CoordDimensions {
    fn into(self) -> u32 {
        match self {
            #[cfg(not(feature = "v3_12_0"))]
            CoordDimensions::OneD => 1,
            CoordDimensions::TwoD => 2,
            CoordDimensions::ThreeD => 3,
            #[cfg(feature = "v3_12_0")]
            CoordDimensions::FourD => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum OutputDimension {
    TwoD,
    ThreeD,
    #[cfg(feature = "v3_12_0")]
    FourD,
}

impl TryFrom<c_int> for OutputDimension {
    type Error = crate::error::Error;

    fn try_from(dimensions: c_int) -> Result<Self, Self::Error> {
        match dimensions {
            2 => Ok(OutputDimension::TwoD),
            3 => Ok(OutputDimension::ThreeD),
            #[cfg(feature = "v3_12_0")]
            4 => Ok(OutputDimension::FourD),
            #[cfg(not(feature = "v3_12_0"))]
            _ => Err(Self::Error::GenericError("dimension must be 2 or 3".into())),
            #[cfg(feature = "v3_12_0")]
            _ => Err(Self::Error::GenericError(
                "dimension must be 2, 3 or 4".into(),
            )),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for OutputDimension {
    fn into(self) -> c_int {
        match self {
            OutputDimension::TwoD => 2,
            OutputDimension::ThreeD => 3,
            #[cfg(feature = "v3_12_0")]
            OutputDimension::FourD => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

impl TryFrom<c_int> for ByteOrder {
    type Error = crate::error::Error;

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
    #[cfg(feature = "v3_13_0")]
    CircularString,
    #[cfg(feature = "v3_13_0")]
    CompoundCurve,
    #[cfg(feature = "v3_13_0")]
    CurvePolygon,
    #[cfg(feature = "v3_13_0")]
    MultiCurve,
    #[cfg(feature = "v3_13_0")]
    MultiSurface,
}

impl TryFrom<c_int> for GeometryTypes {
    type Error = crate::error::Error;

    fn try_from(type_id: c_int) -> Result<Self, Self::Error> {
        match type_id {
            0 => Ok(GeometryTypes::Point),
            1 => Ok(GeometryTypes::LineString),
            2 => Ok(GeometryTypes::LinearRing),
            3 => Ok(GeometryTypes::Polygon),
            4 => Ok(GeometryTypes::MultiPoint),
            5 => Ok(GeometryTypes::MultiLineString),
            6 => Ok(GeometryTypes::MultiPolygon),
            7 => Ok(GeometryTypes::GeometryCollection),
            #[cfg(feature = "v3_13_0")]
            8 => Ok(GeometryTypes::CircularString),
            #[cfg(feature = "v3_13_0")]
            9 => Ok(GeometryTypes::CompoundCurve),
            #[cfg(feature = "v3_13_0")]
            10 => Ok(GeometryTypes::CurvePolygon),
            #[cfg(feature = "v3_13_0")]
            11 => Ok(GeometryTypes::MultiCurve),
            #[cfg(feature = "v3_13_0")]
            12 => Ok(GeometryTypes::MultiSurface),
            _ => Err(crate::Error::GenericError(
                "invalid geometry type id".into(),
            )),
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
            #[cfg(feature = "v3_13_0")]
            GeometryTypes::CircularString => 8,
            #[cfg(feature = "v3_13_0")]
            GeometryTypes::CompoundCurve => 9,
            #[cfg(feature = "v3_13_0")]
            GeometryTypes::CurvePolygon => 10,
            #[cfg(feature = "v3_13_0")]
            GeometryTypes::MultiCurve => 11,
            #[cfg(feature = "v3_13_0")]
            GeometryTypes::MultiSurface => 12,
        }
    }
}

impl GeometryTypes {
    #[cfg(not(feature = "v3_13_0"))]
    pub fn is_surface(self) -> bool {
        matches!(self, Self::Polygon)
    }
    #[cfg(feature = "v3_13_0")]
    pub fn is_surface(self) -> bool {
        matches!(self, Self::Polygon | Self::CurvePolygon)
    }
    #[cfg(not(feature = "v3_13_0"))]
    pub fn is_curve(self) -> bool {
        matches!(self, Self::LineString | Self::LinearRing)
    }
    #[cfg(feature = "v3_13_0")]
    pub fn is_curve(self) -> bool {
        matches!(
            self,
            Self::LineString | Self::LinearRing | Self::CircularString
        )
    }
    #[cfg(not(feature = "v3_13_0"))]
    pub fn is_collection(self) -> bool {
        matches!(
            self,
            Self::GeometryCollection
                | Self::MultiPoint
                | Self::MultiLineString
                | Self::MultiPolygon
        )
    }
    #[cfg(feature = "v3_13_0")]
    pub fn is_collection(self) -> bool {
        matches!(
            self,
            Self::GeometryCollection
                | Self::MultiPoint
                | Self::MultiLineString
                | Self::MultiPolygon
                | Self::MultiCurve
                | Self::MultiSurface
        )
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
    type Error = crate::error::Error;

    fn try_from(orientation: c_int) -> Result<Self, Self::Error> {
        match orientation {
            -1 => Ok(Orientation::CounterClockwise),
            0 => Ok(Orientation::Clockwise),
            1 => Ok(Orientation::Colinear),
            _ => Err(Self::Error::GenericError("value must be -1, 0 or 1".into())),
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
    #[cfg(feature = "v3_14_0")]
    M,
}

impl TryFrom<size_t> for Ordinate {
    type Error = crate::error::Error;

    fn try_from(ordinate: size_t) -> Result<Self, Self::Error> {
        match ordinate {
            0 => Ok(Ordinate::X),
            1 => Ok(Ordinate::Y),
            2 => Ok(Ordinate::Z),
            #[cfg(feature = "v3_14_0")]
            3 => Ok(Ordinate::M),
            #[cfg(not(feature = "v3_14_0"))]
            _ => Err(Self::Error::GenericError(
                "ordinate value must be >= 0 and <= 2".into(),
            )),
            #[cfg(feature = "v3_14_0")]
            _ => Err(Self::Error::GenericError(
                "ordinate value must be >= 0 and <= 3".into(),
            )),
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
            #[cfg(feature = "v3_14_0")]
            Ordinate::M => 3,
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
    type Error = crate::error::Error;

    fn try_from(order: c_int) -> Result<Self, Self::Error> {
        match order {
            0 => Ok(Precision::ValidOutput),
            1 => Ok(Precision::NoTopo),
            2 => Ok(Precision::KeepCollapsed),
            _ => Err(Self::Error::GenericError("Unknown precision type".into())),
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
    type Error = crate::error::Error;

    fn try_from(join_style: c_int) -> Result<Self, Self::Error> {
        match join_style {
            1 => Ok(JoinStyle::Round),
            2 => Ok(JoinStyle::Mitre),
            3 => Ok(JoinStyle::Bevel),
            _ => Err(Self::Error::GenericError("Unknown join style".into())),
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
    type Error = crate::error::Error;

    fn try_from(cap_style: c_int) -> Result<Self, Self::Error> {
        match cap_style {
            1 => Ok(CapStyle::Round),
            2 => Ok(CapStyle::Flat),
            3 => Ok(CapStyle::Square),
            _ => Err(Self::Error::GenericError("Unknown cap style".into())),
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
