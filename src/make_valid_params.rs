use crate::context_handle::with_context;
use crate::functions::{errcheck, nullcheck};
use crate::traits::as_raw_mut_impl;
use crate::{AsRaw, AsRawMut, GResult, MakeValidMethod};
use geos_sys::*;
use std::ptr::NonNull;

/// Contains the parameters which describe how a [Geometry](crate::Geometry) should be validated using [`make_valid_with_params`](crate::Geom::make_valid_with_params)
pub struct MakeValidParams {
    ptr: NonNull<GEOSMakeValidParams>,
}

/// Build options for a [`MakeValidParams`] object
#[derive(Default)]
pub struct MakeValidParamsBuilder {
    method: Option<MakeValidMethod>,
    keep_collapsed: Option<bool>,
}

impl MakeValidParams {
    pub fn new() -> GResult<Self> {
        with_context(|ctx| unsafe {
            let ptr = nullcheck!(GEOSMakeValidParams_create_r(ctx.as_raw()))?;
            Ok(Self { ptr })
        })
    }

    pub fn builder() -> MakeValidParamsBuilder {
        MakeValidParamsBuilder::default()
    }

    /// Sets the method to use for making the geometry valid.
    ///
    /// - [`MakeValidMethod::Linework`]: Uses the original algorithm which combines
    ///   geometry components using the linework from the input geometry.
    /// - [`MakeValidMethod::Structure`]: Rebuilds valid geometries by determining
    ///   rings and polygonizing them. Often produces better results on strongly
    ///   invalid inputs.
    pub fn set_method(&mut self, method: MakeValidMethod) -> GResult<()> {
        with_context(|ctx| unsafe {
            errcheck!(GEOSMakeValidParams_setMethod_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
                method.into(),
            ))?;
            Ok(())
        })
    }

    /// Sets whether to preserve collapsed geometries.
    ///
    /// When set to `true`, collapsed geometries (e.g., a triangle collapsing to a line)
    /// are preserved in the output. When `false`, they are removed.
    ///
    /// Default is `false`.
    pub fn set_keep_collapsed(&mut self, keep_collapsed: bool) -> GResult<()> {
        with_context(|ctx| unsafe {
            errcheck!(GEOSMakeValidParams_setKeepCollapsed_r(
                ctx.as_raw(),
                self.as_raw_mut_override(),
                keep_collapsed.into(),
            ))?;
            Ok(())
        })
    }
}

unsafe impl Send for MakeValidParams {}
unsafe impl Sync for MakeValidParams {}

impl Drop for MakeValidParams {
    fn drop(&mut self) {
        with_context(|ctx| unsafe {
            GEOSMakeValidParams_destroy_r(ctx.as_raw(), self.as_raw_mut());
        });
    }
}

as_raw_mut_impl!(MakeValidParams, GEOSMakeValidParams);

impl MakeValidParamsBuilder {
    pub const fn method(mut self, method: MakeValidMethod) -> Self {
        self.method = Some(method);
        self
    }

    pub const fn keep_collapsed(mut self, keep_collapsed: bool) -> Self {
        self.keep_collapsed = Some(keep_collapsed);
        self
    }

    pub fn build(self) -> GResult<MakeValidParams> {
        let mut params = MakeValidParams::new()?;
        if let Some(method) = self.method {
            params.set_method(method)?;
        }
        if let Some(keep_collapsed) = self.keep_collapsed {
            params.set_keep_collapsed(keep_collapsed)?;
        }
        Ok(params)
    }
}
