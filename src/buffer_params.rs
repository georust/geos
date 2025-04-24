use crate::context_handle::with_context;
use crate::enums::CapStyle;
use crate::functions::{errcheck, nullcheck};
use crate::traits::as_raw_mut_impl;
use crate::{AsRaw, AsRawMut, GResult, JoinStyle};
use geos_sys::*;
use std::ptr::NonNull;

/// Contains the parameters which describe how a [Geometry](crate::Geometry) buffer should be constructed using [`buffer_with_params`](crate::Geom::buffer_with_params)
pub struct BufferParams {
    ptr: NonNull<GEOSBufferParams>,
}

/// Build options for a [`BufferParams`] object
#[derive(Default)]
pub struct BufferParamsBuilder {
    end_cap_style: Option<CapStyle>,
    join_style: Option<JoinStyle>,
    mitre_limit: Option<f64>,
    quadrant_segments: Option<i32>,
    single_sided: Option<bool>,
}

impl BufferParams {
    pub fn new() -> GResult<BufferParams> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSBufferParams_create_r(ctx.as_raw()))?;
            Ok(BufferParams { ptr })
        })
    }

    pub fn builder() -> BufferParamsBuilder {
        BufferParamsBuilder::default()
    }

    /// Specifies the end cap style of the generated buffer.
    pub fn set_end_cap_style(&mut self, style: CapStyle) -> GResult<()> {
        with_context(|ctx| unsafe {
            errcheck!(GEOSBufferParams_setEndCapStyle_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
                style.into(),
            ))?;
            Ok(())
        })
    }

    /// Sets the join style for outside (reflex) corners between line segments.
    pub fn set_join_style(&mut self, style: JoinStyle) -> GResult<()> {
        with_context(|ctx| unsafe {
            errcheck!(GEOSBufferParams_setJoinStyle_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
                style.into(),
            ))?;
            Ok(())
        })
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
        with_context(|ctx| unsafe {
            errcheck!(GEOSBufferParams_setMitreLimit_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
                limit
            ))?;
            Ok(())
        })
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
        with_context(|ctx| unsafe {
            errcheck!(GEOSBufferParams_setQuadrantSegments_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
                quadsegs as _,
            ))?;
            Ok(())
        })
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
        with_context(|ctx| unsafe {
            errcheck!(GEOSBufferParams_setSingleSided_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
                is_single_sided.into(),
            ))?;
            Ok(())
        })
    }
}

unsafe impl Send for BufferParams {}
unsafe impl Sync for BufferParams {}

impl Drop for BufferParams {
    fn drop(&mut self) {
        with_context(|ctx| unsafe { GEOSBufferParams_destroy_r(ctx.as_raw(), self.as_raw_mut()) });
    }
}

as_raw_mut_impl!(BufferParams, GEOSBufferParams);

impl BufferParamsBuilder {
    pub fn end_cap_style(mut self, style: CapStyle) -> BufferParamsBuilder {
        self.end_cap_style = Some(style);
        self
    }
    pub fn join_style(mut self, style: JoinStyle) -> BufferParamsBuilder {
        self.join_style = Some(style);
        self
    }
    pub fn mitre_limit(mut self, limit: f64) -> BufferParamsBuilder {
        self.mitre_limit = Some(limit);
        self
    }
    pub fn quadrant_segments(mut self, quadsegs: i32) -> BufferParamsBuilder {
        self.quadrant_segments = Some(quadsegs);
        self
    }
    pub fn single_sided(mut self, is_single_sided: bool) -> BufferParamsBuilder {
        self.single_sided = Some(is_single_sided);
        self
    }
    pub fn build(self) -> GResult<BufferParams> {
        let mut params = BufferParams::new()?;
        if let Some(style) = self.end_cap_style {
            params.set_end_cap_style(style)?;
        }
        if let Some(style) = self.join_style {
            params.set_join_style(style)?;
        }
        if let Some(limit) = self.mitre_limit {
            params.set_mitre_limit(limit)?;
        }
        if let Some(quad_segs) = self.quadrant_segments {
            params.set_quadrant_segments(quad_segs)?;
        }
        if let Some(is_single_sided) = self.single_sided {
            params.set_single_sided(is_single_sided)?;
        }
        Ok(params)
    }
}
