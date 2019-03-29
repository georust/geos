use error::{Error, GResult};
use ffi::*;
use functions::*;
use libc::{c_double, c_uint};
use std::ptr::NonNull;

pub struct CoordSeq(NonNull<GEOSCoordSequence>);

impl Drop for CoordSeq {
    fn drop(&mut self) {
        unsafe { GEOSCoordSeq_destroy(self.0.as_mut()) };
    }
}

impl Clone for CoordSeq {
    fn clone(&self) -> CoordSeq {
        CoordSeq(NonNull::new(unsafe { GEOSCoordSeq_clone(self.0.as_ref()) }).unwrap())
    }
}

impl CoordSeq {
    pub fn new(size: u32, dims: u32) -> CoordSeq {
        initialize();
        CoordSeq(
            NonNull::new(unsafe { GEOSCoordSeq_create(size as c_uint, dims as c_uint) }).unwrap(),
        )
    }

    pub(crate) fn as_raw(&self) -> *const GEOSCoordSequence {
        unsafe { self.0.as_ref() }
    }

    pub(crate) unsafe fn new_from_raw(c_obj: *mut GEOSCoordSequence) -> GResult<CoordSeq> {
        NonNull::new(c_obj)
            .ok_or(Error::NoConstructionFromNullPtr)
            .map(CoordSeq)
    }

    pub fn set_x(&mut self, idx: u32, val: f64) -> GResult<()> {
        let ret_val = unsafe { GEOSCoordSeq_setX(self.0.as_mut(), idx as c_uint, val as c_double) };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set x for coord".into()))
        } else {
            Ok(())
        }
    }

    pub fn set_y(&mut self, idx: u32, val: f64) -> GResult<()> {
        let ret_val = unsafe { GEOSCoordSeq_setY(self.0.as_mut(), idx as c_uint, val as c_double) };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set y for coord".into()))
        } else {
            Ok(())
        }
    }

    pub fn set_z(&mut self, idx: u32, val: f64) -> GResult<()> {
        let ret_val = unsafe { GEOSCoordSeq_setZ(self.0.as_mut(), idx as c_uint, val as c_double) };
        if ret_val == 0 {
            Err(Error::GeosError("impossible to set z for coord".into()))
        } else {
            Ok(())
        }
    }

    pub fn get_x(&self, idx: u32) -> GResult<f64> {
        let mut n = 0.0 as c_double;
        let ret_val = unsafe { GEOSCoordSeq_getX(self.0.as_ref(), idx as c_uint, &mut n) };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn get_y(&self, idx: u32) -> GResult<f64> {
        let mut n = 0.0 as c_double;
        let ret_val = unsafe { GEOSCoordSeq_getY(self.0.as_ref(), idx as c_uint, &mut n) };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn get_z(&self, idx: u32) -> GResult<f64> {
        let mut n = 0.0 as c_double;
        let ret_val = unsafe { GEOSCoordSeq_getZ(self.0.as_ref(), idx as c_uint, &mut n) };
        if ret_val == 0 {
            Err(Error::GeosError("getting coordinates from CoordSeq".into()))
        } else {
            Ok(n as f64)
        }
    }

    pub fn len(&self) -> GResult<usize> {
        let mut n = 0 as c_uint;
        let ret_val = unsafe { GEOSCoordSeq_getSize(self.0.as_ref(), &mut n) };
        if ret_val == 0 {
            Err(Error::GeosError("getting size from CoordSeq".into()))
        } else {
            Ok(n as usize)
        }
    }
}
