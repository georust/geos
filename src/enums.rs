use libc::c_int;

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
