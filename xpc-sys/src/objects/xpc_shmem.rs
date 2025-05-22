use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_object::XPCObject;
use crate::{rs_strerror, xpc_shmem_create};
use mach2::port::mach_port_t;
use mach2::traps::mach_task_self;
use mach2::vm::{mach_vm_allocate, mach_vm_deallocate};
use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t};
use std::ffi::c_void;
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::sync::Arc;

/// Wrapper around vm_allocate() vm_deallocate() with an XPCObject
/// member of XPC type _xpc_type_shmem
#[derive(Debug, Clone)]
pub struct XPCShmem {
    pub task: mach_port_t,
    pub size: mach_vm_size_t,
    pub region: *mut c_void,
    pub xpc_object: Arc<XPCObject>,
}

unsafe impl Send for XPCShmem {}

impl XPCShmem {
    /// Allocate a region of memory of vm_size_t & flags, then wrap in a XPC Object
    #[must_use]
    pub fn new(task: mach_port_t, size: mach_vm_size_t, flags: c_int) -> Result<XPCShmem, XPCError> {
        let mut region: *mut c_void = null_mut();
        let err = unsafe {
            mach_vm_allocate(
                task,
                &mut region as *const _ as *mut mach_vm_address_t,
                size,
                flags,
            )
        };

        if err > 0 {
            Err(XPCError::IOError(rs_strerror(err)))
        } else {
            let xpc_object: XPCObject =
                unsafe { xpc_shmem_create(region, size as usize).into() };

            log::info!(
                "XPCShmem new (region: {:p}, xpc_object_t {:p})",
                region,
                xpc_object.as_ptr()
            );

            Ok(XPCShmem {
                task,
                size,
                region,
                xpc_object: xpc_object.into(),
            })
        }
    }

    /// new() with _mach_task_self
    /// https://web.mit.edu/darwin/src/modules/xnu/osfmk/man/mach_task_self.html
    #[must_use]
    pub fn new_task_self(size: mach_vm_size_t, flags: c_int) -> Result<XPCShmem, XPCError> {
        unsafe { Self::new(mach_task_self(), size, flags) }
    }
}

impl Drop for XPCShmem {
    fn drop(&mut self) {
        let XPCShmem {
            size,
            task,
            region,
            xpc_object,
        } = self;
        log::info!(
            "XPCShmem drop (region: {:p}, xpc_object_t {:p})",
            region,
            xpc_object.as_ptr()
        );

        let ok = unsafe { mach_vm_deallocate(*task, *region as mach_vm_address_t, *size) };

        if ok != 0 {
            panic!("shmem won't drop (vm_deallocate errno {})", ok);
        }
    }
}
