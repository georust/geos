use libc::{c_int, size_t};

use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum CoordDimensions {
    TwoD,
    ThreeD,
    #[cfg(feature = "v3_12_0")]
    FourD,
}

impl TryFrom<c_int> for CoordDimensions {
    type Error = crate::error::Error;

    fn try_from(dimensions: c_int) -> Result<Self, Self::Error> {
        match dimensions {
            2 => Ok(Self::TwoD),
            3 => Ok(Self::ThreeD),
            #[cfg(feature = "v3_12_0")]
            4 => Ok(Self::FourD),
            #[cfg(not(feature = "v3_12_0"))]
            _ => Err(Self::Error::GenericError(
                "dimensions must be 2 or 3".into(),
            )),
            #[cfg(feature = "v3_12_0")]
            _ => Err(Self::Error::GenericError(
                "dimensions must be 2, 3 or 4".into(),
            )),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for CoordDimensions {
    fn into(self) -> c_int {
        match self {
            Self::TwoD => 2,
            Self::ThreeD => 3,
            #[cfg(feature = "v3_12_0")]
            Self::FourD => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum CoordType {
    XY,
    XYZ,
    #[cfg(feature = "v3_12_0")]
    XYZM,
    #[cfg(feature = "v3_12_0")]
    XYM,
}

impl CoordType {
    #[cfg(not(feature = "v3_12_0"))]
    pub const fn has_z(&self) -> bool {
        matches!(self, Self::XYZ)
    }
    #[cfg(feature = "v3_12_0")]
    pub const fn has_z(&self) -> bool {
        matches!(self, Self::XYZ | Self::XYZM)
    }
    #[cfg(not(feature = "v3_12_0"))]
    pub const fn has_m(&self) -> bool {
        false
    }
    #[cfg(feature = "v3_12_0")]
    pub const fn has_m(&self) -> bool {
        matches!(self, Self::XYM | Self::XYZM)
    }
}

impl TryFrom<(bool, bool)> for CoordType {
    type Error = crate::error::Error;

    fn try_from((has_z, has_m): (bool, bool)) -> Result<Self, Self::Error> {
        match (has_z, has_m) {
            (false, false) => Ok(Self::XY),
            (true, false) => Ok(Self::XYZ),
            #[cfg(feature = "v3_12_0")]
            (false, true) => Ok(Self::XYM),
            #[cfg(feature = "v3_12_0")]
            (true, true) => Ok(Self::XYZM),
            #[cfg(not(feature = "v3_12_0"))]
            _ => Err(Self::Error::GenericError("unsupported dimension".into())),
        }
    }
}

impl TryFrom<u32> for CoordType {
    type Error = crate::error::Error;

    fn try_from(dimensions: u32) -> Result<Self, Self::Error> {
        match dimensions {
            2 => Ok(Self::XY),
            3 => Ok(Self::XYZ),
            #[cfg(feature = "v3_12_0")]
            4 => Ok(Self::XYZM),
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

impl From<CoordType> for u32 {
    fn from(c: CoordType) -> Self {
        match c {
            CoordType::XY => 2,
            CoordType::XYZ => 3,
            #[cfg(feature = "v3_12_0")]
            CoordType::XYM => 3,
            #[cfg(feature = "v3_12_0")]
            CoordType::XYZM => 4,
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
            0 => Ok(Self::BigEndian),
            _ => Ok(Self::LittleEndian),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for ByteOrder {
    fn into(self) -> c_int {
        match self {
            Self::BigEndian => 0,
            Self::LittleEndian => 1,
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
            0 => Ok(Self::Point),
            1 => Ok(Self::LineString),
            2 => Ok(Self::LinearRing),
            3 => Ok(Self::Polygon),
            4 => Ok(Self::MultiPoint),
            5 => Ok(Self::MultiLineString),
            6 => Ok(Self::MultiPolygon),
            7 => Ok(Self::GeometryCollection),
            #[cfg(feature = "v3_13_0")]
            8 => Ok(Self::CircularString),
            #[cfg(feature = "v3_13_0")]
            9 => Ok(Self::CompoundCurve),
            #[cfg(feature = "v3_13_0")]
            10 => Ok(Self::CurvePolygon),
            #[cfg(feature = "v3_13_0")]
            11 => Ok(Self::MultiCurve),
            #[cfg(feature = "v3_13_0")]
            12 => Ok(Self::MultiSurface),
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
            Self::Point => 0,
            Self::LineString => 1,
            Self::LinearRing => 2,
            Self::Polygon => 3,
            Self::MultiPoint => 4,
            Self::MultiLineString => 5,
            Self::MultiPolygon => 6,
            Self::GeometryCollection => 7,
            #[cfg(feature = "v3_13_0")]
            Self::CircularString => 8,
            #[cfg(feature = "v3_13_0")]
            Self::CompoundCurve => 9,
            #[cfg(feature = "v3_13_0")]
            Self::CurvePolygon => 10,
            #[cfg(feature = "v3_13_0")]
            Self::MultiCurve => 11,
            #[cfg(feature = "v3_13_0")]
            Self::MultiSurface => 12,
        }
    }
}

impl GeometryTypes {
    #[cfg(not(feature = "v3_13_0"))]
    pub const fn is_surface(self) -> bool {
        matches!(self, Self::Polygon)
    }
    #[cfg(feature = "v3_13_0")]
    pub const fn is_surface(self) -> bool {
        matches!(self, Self::Polygon | Self::CurvePolygon)
    }
    #[cfg(not(feature = "v3_13_0"))]
    pub const fn is_curve(self) -> bool {
        matches!(self, Self::LineString | Self::LinearRing)
    }
    #[cfg(feature = "v3_13_0")]
    pub const fn is_curve(self) -> bool {
        matches!(
            self,
            Self::LineString | Self::LinearRing | Self::CircularString
        )
    }
    #[cfg(not(feature = "v3_13_0"))]
    pub const fn is_collection(self) -> bool {
        matches!(
            self,
            Self::GeometryCollection
                | Self::MultiPoint
                | Self::MultiLineString
                | Self::MultiPolygon
        )
    }
    #[cfg(feature = "v3_13_0")]
    pub const fn is_collection(self) -> bool {
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
            -1 => Ok(Self::CounterClockwise),
            0 => Ok(Self::Clockwise),
            1 => Ok(Self::Colinear),
            _ => Err(Self::Error::GenericError("value must be -1, 0 or 1".into())),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for Orientation {
    fn into(self) -> c_int {
        match self {
            Self::CounterClockwise => -1,
            Self::Clockwise => 0,
            Self::Colinear => 1,
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
            0 => Ok(Self::X),
            1 => Ok(Self::Y),
            2 => Ok(Self::Z),
            #[cfg(feature = "v3_14_0")]
            3 => Ok(Self::M),
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
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
            #[cfg(feature = "v3_14_0")]
            Self::M => 3,
        }
    }
}

#[cfg(feature = "v3_6_0")]
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Precision {
    ValidOutput,
    NoTopo,
    KeepCollapsed,
}

#[cfg(feature = "v3_6_0")]
impl TryFrom<c_int> for Precision {
    type Error = crate::error::Error;

    fn try_from(order: c_int) -> Result<Self, Self::Error> {
        match order {
            0 => Ok(Self::ValidOutput),
            1 => Ok(Self::NoTopo),
            2 => Ok(Self::KeepCollapsed),
            _ => Err(Self::Error::GenericError("Unknown precision type".into())),
        }
    }
}

#[cfg(feature = "v3_6_0")]
#[allow(clippy::from_over_into)]
impl Into<c_int> for Precision {
    fn into(self) -> c_int {
        match self {
            Self::ValidOutput => 0,
            Self::NoTopo => 1,
            Self::KeepCollapsed => 2,
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
            1 => Ok(Self::Round),
            2 => Ok(Self::Mitre),
            3 => Ok(Self::Bevel),
            _ => Err(Self::Error::GenericError("Unknown join style".into())),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for JoinStyle {
    fn into(self) -> c_int {
        match self {
            Self::Round => 1,
            Self::Mitre => 2,
            Self::Bevel => 3,
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
            1 => Ok(Self::Round),
            2 => Ok(Self::Flat),
            3 => Ok(Self::Square),
            _ => Err(Self::Error::GenericError("Unknown cap style".into())),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<c_int> for CapStyle {
    fn into(self) -> c_int {
        match self {
            Self::Round => 1,
            Self::Flat => 2,
            Self::Square => 3,
        }
    }
}

/// Validation methods for a [`Geometry`](crate::Geometry) [`make_valid_with_params`](crate::Geom::make_valid_with_params) operation
#[cfg(feature = "v3_10_0")]
#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum MakeValidMethod {
    /// The ‘linework’ algorithm tries to preserve every edge and vertex in the input.
    #[default]
    Linework,
    /// The ‘structure’ algorithm tries to reason from the structure of the input to find the ‘correct’ repair
    Structure,
}

#[cfg(feature = "v3_10_0")]
impl TryFrom<u32> for MakeValidMethod {
    type Error = crate::error::Error;

    fn try_from(method: u32) -> Result<Self, Self::Error> {
        match method {
            0 => Ok(Self::Linework),
            1 => Ok(Self::Structure),
            _ => Err(Self::Error::GenericError(
                "Unknown make valid method".into(),
            )),
        }
    }
}

#[cfg(feature = "v3_10_0")]
#[allow(clippy::from_over_into)]
impl Into<u32> for MakeValidMethod {
    fn into(self) -> u32 {
        match self {
            Self::Linework => 0,
            Self::Structure => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum DimensionType {
    Point,
    Curve,
    Surface,
}

impl TryFrom<c_int> for DimensionType {
    type Error = crate::error::Error;

    fn try_from(cap_style: c_int) -> Result<Self, Self::Error> {
        match cap_style {
            0 => Ok(Self::Point),
            1 => Ok(Self::Curve),
            2 => Ok(Self::Surface),
            _ => Err(Self::Error::GenericError("Unknown dimension type".into())),
        }
    }
}
