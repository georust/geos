use ffi::GEOSContextHandle_t;

pub trait AsRaw {
    type RawType;

    fn as_raw(&self) -> Self::RawType;
}

pub trait ContextHandling {
    type Context;

    fn get_raw_context(&self) -> GEOSContextHandle_t;
    fn clone_context(&self) -> Self::Context;
}

pub trait ContextInteractions {
    type Context;

    fn set_context_handle(&mut self, context: Self::Context);
    fn get_context_handle(&self) -> &Self::Context;
}
