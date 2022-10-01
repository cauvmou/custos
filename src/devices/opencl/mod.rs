use std::ffi::c_void;

pub use cl_device::*;
pub use cl_devices::*;
pub use kernel_cache::*;
pub use kernel_enqueue::*;

pub mod api;
pub mod cl_device;
pub mod cl_devices;
mod kernel_cache;
mod kernel_enqueue;
#[cfg(unified_cl)]
mod unified;

#[cfg(unified_cl)]
pub use unified::*;

use crate::{Buffer, CDatatype, PtrType};

use self::api::release_mem_object;

pub struct CLPtr {
    pub ptr: *mut c_void,
    pub host_ptr: *mut u8,
}

impl PtrType for CLPtr {
    unsafe fn alloc<T>(alloc: impl crate::Alloc, len: usize) -> Self {
        let ptrs = alloc.alloc(len);
        CLPtr { 
            ptr: ptrs.1,
            host_ptr: ptrs.0
        }
    }

    unsafe fn dealloc<T>(&mut self, _len: usize) {
        release_mem_object(self.ptr).unwrap();
    }
}

/// Sets the elements of an OpenCL Buffer to zero.
/// # Example
/// ```
/// use custos::{OpenCL, Buffer, VecRead, opencl::cl_clear};
///
/// fn main() -> Result<(), custos::Error> {
///     let device = OpenCL::new(0)?;
///     let mut lhs = Buffer::<i16, _>::from((&device, [15, 30, 21, 5, 8]));
///     assert_eq!(device.read(&lhs), vec![15, 30, 21, 5, 8]);
///
///     cl_clear(&device, &mut lhs);
///     assert_eq!(device.read(&lhs), vec![0; 5]);
///     Ok(())
/// }
/// ```
pub fn cl_clear<T: CDatatype>(device: &OpenCL, lhs: &mut Buffer<T, OpenCL>) -> crate::Result<()> {
    let src = format!(
        "
        __kernel void clear(__global {datatype}* self) {{
            size_t id = get_global_id(0);
            self[id] = 0;
        }}
    ",
        datatype = T::as_c_type_str()
    );

    let gws = [lhs.len, 0, 0];
    enqueue_kernel(device, &src, gws, None, &[lhs])?;
    Ok(())
}
