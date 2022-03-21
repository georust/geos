use std::ffi::c_void;
use std::marker::PhantomData;
use std::sync::Arc;

use geos_sys::*;

use crate::context_handle::PtrWrap;
use crate::ContextHandling;
use crate::{AsRaw, AsRawMut, GResult};
use crate::{ContextHandle, Geom};

pub trait SpatialIndex<'a, I> {
    fn insert<'b, G: Geom<'b>>(&mut self, geometry: &G, item: I);

    fn query<'b, G: Geom<'b>, V: FnMut(&I)>(&self, geometry: &G, visitor: V);
}

pub struct STRtree<'a, I> {
    pub(crate) ptr: PtrWrap<*mut GEOSSTRtree>,
    context: Arc<ContextHandle<'a>>,
    item_type: PhantomData<I>,
}

impl<'a, I> STRtree<'a, I> {
    pub fn with_capacity(node_capacity: usize) -> GResult<STRtree<'a, I>> {
        match ContextHandle::init_e(Some("STRtree::with_capacity")) {
            Ok(context_handle) => unsafe {
                let ptr = GEOSSTRtree_create_r(context_handle.as_raw(), node_capacity);
                Ok(STRtree {
                    ptr: PtrWrap(ptr),
                    context: Arc::new(context_handle),
                    item_type: PhantomData,
                })
            },
            Err(e) => Err(e),
        }
    }

    pub fn iterate<V>(&self, visitor: V)
    where
        V: FnMut(&I),
    {
        unsafe {
            let (closure, callback) = unpack_closure(&visitor);
            GEOSSTRtree_iterate_r(self.get_raw_context(), *self.ptr, Some(callback), closure);
        }
    }
}

impl<'a, I> SpatialIndex<'a, I> for STRtree<'a, I> {
    fn insert<'b, G: Geom<'b>>(&mut self, geometry: &G, item: I) {
        unsafe {
            GEOSSTRtree_insert_r(
                self.get_raw_context(),
                *self.ptr,
                geometry.as_raw(),
                Box::into_raw(Box::new(item)) as *mut c_void,
            );
        }
    }

    fn query<'b, G: Geom<'b>, V: FnMut(&I)>(&self, geometry: &G, visitor: V) {
        unsafe {
            let (closure, callback) = unpack_closure(&visitor);
            GEOSSTRtree_query_r(
                self.get_raw_context(),
                *self.ptr,
                geometry.as_raw(),
                Some(callback),
                closure,
            );
        }
    }
}

impl<'a, I> AsRaw for STRtree<'a, I> {
    type RawType = GEOSSTRtree;

    fn as_raw(&self) -> *const Self::RawType {
        *self.ptr
    }
}

impl<'a, I> AsRawMut for STRtree<'a, I> {
    type RawType = GEOSSTRtree;

    unsafe fn as_raw_mut_override(&self) -> *mut Self::RawType {
        *self.ptr
    }
}

impl<'a, I> ContextHandling for STRtree<'a, I> {
    type Context = Arc<ContextHandle<'a>>;

    fn get_raw_context(&self) -> GEOSContextHandle_t {
        self.context.as_raw()
    }

    fn clone_context(&self) -> Arc<ContextHandle<'a>> {
        Arc::clone(&self.context)
    }
}

impl<I> Drop for STRtree<'_, I> {
    fn drop(&mut self) {
        unsafe extern "C" fn callback<I>(item: *mut c_void, _data: *mut c_void) {
            drop(Box::from_raw(item as *mut I));
        }

        unsafe {
            GEOSSTRtree_iterate_r(
                self.get_raw_context(),
                *self.ptr,
                Some(callback::<I>),
                std::ptr::null_mut(),
            );
            GEOSSTRtree_destroy_r(self.get_raw_context(), *self.ptr);
        }
    }
}

unsafe fn unpack_closure<F, I>(
    closure: &F,
) -> (*mut c_void, extern "C" fn(*mut c_void, *mut c_void))
where
    F: FnMut(&I),
{
    extern "C" fn trampoline<F, I>(item: *mut c_void, data: *mut c_void)
    where
        F: FnMut(&I),
    {
        unsafe {
            let closure: &mut F = &mut *(data as *mut F);
            (*closure)(&mut *(item as *mut I));
        }
    }

    (closure as *const F as *mut c_void, trampoline::<F, I>)
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{Geometry, STRtree, SpatialIndex};

    #[test]
    fn test_strtree() {
        let mut tree = STRtree::<&str>::with_capacity(10).unwrap();

        let point = Geometry::new_from_wkt("POINT(5 5)").unwrap();
        let line = Geometry::new_from_wkt("LINESTRING (0 0, 10 0)").unwrap();
        let polygon = Geometry::new_from_wkt("POLYGON((2 2, 8 2, 8 8, 2 8, 2 2))").unwrap();

        tree.insert(&point, "Point");
        tree.insert(&line, "Line");
        tree.insert(&polygon, "Polygon");

        // Test iterate

        let mut items = HashSet::<&str>::new();
        tree.iterate(|item| {
            items.insert(*item);
        });

        assert_eq!(
            items,
            vec!["Line", "Point", "Polygon"].into_iter().collect()
        );

        // Test query

        items.clear();
        tree.query(&point, |item| {
            items.insert(*item);
        });

        assert_eq!(items, vec!["Point", "Polygon"].into_iter().collect());
    }
}
