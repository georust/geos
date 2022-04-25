use crate::context_handle::PtrWrap;
use crate::enums::CapStyle;
use crate::{
    AsRaw, AsRawMut, ContextHandle, ContextHandling, ContextInteractions, Error, GResult, JoinStyle,
};

use geos_sys::*;
use std::sync::Arc;

pub const DEFAULT_MITRE_LIMIT: f64 = 5.0;
pub const DEFAULT_QUADRANT_SEGMENTS: i32 = 8;
pub const DEFAULT_SINGLE_SIDED: bool = false;

/// Contains the parameters which describe how a [Geometry](crate::Geometry) buffer should be constructed using [buffer_with_params](crate::Geom::buffer_with_params)
pub struct BufferParams<'a> {
    ptr: PtrWrap<*mut GEOSBufferParams>,
    context: Arc<ContextHandle<'a>>,
}

/// Build options for a [`BufferParams`] object
pub struct BufferParamsBuilder {
    end_cap_style: CapStyle,
    join_style: JoinStyle,
    mitre_limit: f64,
    quadrant_segments: i32,
    single_sided: bool,
}

impl Default for BufferParamsBuilder {
    fn default() -> Self {
        BufferParamsBuilder {
            end_cap_style: Default::default(),
            join_style: Default::default(),
            mitre_limit: DEFAULT_MITRE_LIMIT,
            quadrant_segments: DEFAULT_QUADRANT_SEGMENTS,
            single_sided: DEFAULT_SINGLE_SIDED,
        }
    }
}

impl<'a> BufferParams<'a> {
    pub fn new() -> GResult<BufferParams<'a>> {
        match ContextHandle::init_e(Some("BufferParams::new")) {
            Ok(context) => unsafe {
                let ptr = GEOSBufferParams_create_r(context.as_raw());
                Ok(BufferParams {
                    ptr: PtrWrap(ptr),
                    context: Arc::new(context),
                })
            },
            Err(e) => Err(e),
        }
    }

    pub fn builder() -> BufferParamsBuilder {
        BufferParamsBuilder::default()
    }

    /// Specifies the end cap style of the generated buffer.
    pub fn set_end_cap_style(&mut self, style: CapStyle) -> GResult<()> {
        unsafe {
            let ret = GEOSBufferParams_setEndCapStyle_r(
                self.get_raw_context(),
                self.as_raw_mut_override(),
                style.into(),
            );
            if ret == 0 {
                Err(Error::GeosError("GEOSBufferParams_setEndCapStyle_r".into()))
            } else {
                Ok(())
            }
        }
    }

    /// Sets the join style for outside (reflex) corners between line segments.
    pub fn set_join_style(&mut self, style: JoinStyle) -> GResult<()> {
        unsafe {
            let ret = GEOSBufferParams_setJoinStyle_r(
                self.get_raw_context(),
                self.as_raw_mut_override(),
                style.into(),
            );
            if ret == 0 {
                Err(Error::GeosError("GEOSBufferParams_setJoinStyle_r".into()))
            } else {
                Ok(())
            }
        }
    }

    /// Sets the limit on the mitre ratio used for very sharp corners.
    ///
    /// The mitre ratio is the ratio of the distance from the corner
    /// to the end of the mitred offset corner.
    /// When two line segments meet at a sharp angle,
    /// a miter join will extend far beyond the original geometry.
    /// (and in the extreme case will be infinitely far.)
    /// To prevent unreasonable geometry, the mitre limit
    /// allows controlling the maximum length of the join corner.
    /// Corners with a ratio which exceed the limit will be beveled.
    pub fn set_mitre_limit(&mut self, limit: f64) -> GResult<()> {
        unsafe {
            let ret = GEOSBufferParams_setMitreLimit_r(
                self.get_raw_context(),
                self.as_raw_mut_override(),
                limit,
            );
            if ret == 0 {
                Err(Error::GeosError("GEOSBufferParams_setMitreLimit_r".into()))
            } else {
                Ok(())
            }
        }
    }

    /// Sets the number of line segments used to approximate
    /// an angle fillet.
    ///
    /// - If `quadsegs` >= 1, joins are round, and `quadsegs` indicates the number of
    ///   segments to use to approximate a quarter-circle.
    /// - If `quadsegs` = 0, joins are bevelled (flat)
    /// - If `quadSegs` < 0, joins are mitred, and the value of qs
    ///   indicates the mitre ration limit as `mitreLimit = |quadsegs|`
    ///
    /// For round joins, `quadsegs` determines the maximum
    /// error in the approximation to the true buffer curve.
    ///
    /// The default value of 8 gives less than 2% max error in the
    /// buffer distance.
    ///
    /// For a max error of < 1%, use QS = 12.
    /// For a max error of < 0.1%, use QS = 18.
    /// The error is always less than the buffer distance
    /// (in other words, the computed buffer curve is always inside
    ///  the true curve).
    pub fn set_quadrant_segments(&mut self, quadsegs: i32) -> GResult<()> {
        unsafe {
            let ret = GEOSBufferParams_setQuadrantSegments_r(
                self.get_raw_context(),
                self.as_raw_mut_override(),
                quadsegs as _,
            );
            if ret == 0 {
                Err(Error::GeosError(
                    "GEOSBufferParams_setQuadrantSegments_r".into(),
                ))
            } else {
                Ok(())
            }
        }
    }

    /// Sets whether the computed buffer should be single-sided.
    ///
    /// A single-sided buffer is constructed on only one side of each input line.
    ///
    /// The side used is determined by the sign of the buffer distance:
    /// - a positive distance indicates the left-hand side
    /// - a negative distance indicates the right-hand side
    ///
    /// The single-sided buffer of point geometries is the same as the regular buffer.
    ///
    /// The End Cap Style for single-sided buffers is always ignored, and forced to the
    /// equivalent of [`CapStyle::Flat`].
    pub fn set_single_sided(&mut self, is_single_sided: bool) -> GResult<()> {
        unsafe {
            let single_sided = if is_single_sided { 1 } else { 0 };
            let ret = GEOSBufferParams_setSingleSided_r(
                self.get_raw_context(),
                self.as_raw_mut_override(),
                single_sided,
            );
            if ret == 0 {
                Err(Error::GeosError("GEOSBufferParams_setSingleSided_r".into()))
            } else {
                Ok(())
            }
        }
    }
}

unsafe impl<'a> Send for BufferParams<'a> {}
unsafe impl<'a> Sync for BufferParams<'a> {}

impl<'a> Drop for BufferParams<'a> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { GEOSBufferParams_destroy_r(self.get_raw_context(), self.as_raw_mut()) };
        }
    }
}

impl<'a> AsRaw for BufferParams<'a> {
    type RawType = GEOSBufferParams;

    fn as_raw(&self) -> *const Self::RawType {
        *self.ptr
    }
}

impl<'a> AsRawMut for BufferParams<'a> {
    type RawType = GEOSBufferParams;

    unsafe fn as_raw_mut_override(&self) -> *mut Self::RawType {
        *self.ptr
    }
}

impl<'a> ContextInteractions<'a> for BufferParams<'a> {
    fn set_context_handle(&mut self, context: ContextHandle<'a>) {
        self.context = Arc::new(context);
    }

    fn get_context_handle(&self) -> &ContextHandle<'a> {
        &self.context
    }
}

impl<'a> ContextHandling for BufferParams<'a> {
    type Context = Arc<ContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}

impl BufferParamsBuilder {
    pub fn end_cap_style(mut self, style: CapStyle) -> BufferParamsBuilder {
        self.end_cap_style = style;
        self
    }
    pub fn join_style(mut self, style: JoinStyle) -> BufferParamsBuilder {
        self.join_style = style;
        self
    }
    pub fn mitre_limit(mut self, limit: f64) -> BufferParamsBuilder {
        self.mitre_limit = limit;
        self
    }
    pub fn quadrant_segments(mut self, quadsegs: i32) -> BufferParamsBuilder {
        self.quadrant_segments = quadsegs;
        self
    }
    pub fn single_sided(mut self, is_single_sided: bool) -> BufferParamsBuilder {
        self.single_sided = is_single_sided;
        self
    }
    pub fn build(self) -> GResult<BufferParams<'static>> {
        let mut params = BufferParams::new()?;
        params.set_end_cap_style(self.end_cap_style)?;
        params.set_join_style(self.join_style)?;
        params.set_mitre_limit(self.mitre_limit)?;
        params.set_quadrant_segments(self.quadrant_segments)?;
        params.set_single_sided(self.single_sided)?;
        Ok(params)
    }
}
