use crate::object::xpc_error::XPCError;
use crate::object::xpc_object::XPCObject;
use crate::{rs_strerror, xpc_object_t, xpc_retain, xpc_shmem_create, xpc_shmem_map};
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
    pub task: Option<mach_port_t>,
    pub size: mach_vm_size_t,
    pub region: *mut c_void,
    pub xpc_object: Arc<XPCObject>,
}

unsafe impl Send for XPCShmem {}

impl XPCShmem {
    pub fn from_xpc_object(value: XPCObject) -> XPCShmem {
        let mut region: *mut c_void = null_mut();
        let size: mach_vm_size_t =
            unsafe { xpc_shmem_map(value.as_ptr(), &mut region) as mach_vm_size_t };

        XPCShmem {
            task: None,
            region,
            size,
            xpc_object: Arc::new(value),
        }
    }

    pub unsafe fn from_raw(value: xpc_object_t) -> XPCShmem {
        let mut region: *mut c_void = null_mut();
        let size: mach_vm_size_t = unsafe { xpc_shmem_map(value, &mut region) as mach_vm_size_t };

        XPCShmem {
            task: None,
            region,
            size,
            xpc_object: XPCObject::from_raw(value).into(),
        }
    }

    /// Allocate a region of memory of vm_size_t & flags, then wrap in a XPC Object
    pub unsafe fn allocate(
        task: mach_port_t,
        size: mach_vm_size_t,
        flags: c_int,
    ) -> Result<XPCShmem, XPCError> {
        let mut region: *mut c_void = null_mut();
        let err = mach_vm_allocate(
            task,
            &mut region as *const _ as *mut mach_vm_address_t,
            size,
            flags,
        );

        if err > 0 {
            Err(XPCError::IOError(rs_strerror(err)))
        } else {
            let xpc_object: XPCObject =
                unsafe { XPCObject::from_raw(xpc_shmem_create(region, size as usize)) };

            log::info!(
                "XPCShmem new (region: {:p}, xpc_object_t {:p})",
                region,
                xpc_object.as_ptr()
            );

            Ok(XPCShmem {
                task: Some(task),
                size,
                region,
                xpc_object: xpc_object.into(),
            })
        }
    }

    /// Make a new XPCShmem with _mach_task_self
    /// https://web.mit.edu/darwin/src/modules/xnu/osfmk/man/mach_task_self.html
    pub fn allocate_task_self(size: mach_vm_size_t, flags: c_int) -> Result<XPCShmem, XPCError> {
        unsafe { Self::allocate(mach_task_self(), size, flags) }
    }
}

impl From<XPCObject> for XPCShmem {
    fn from(value: XPCObject) -> Self {
        Self::from_xpc_object(value)
    }
}

impl From<&XPCObject> for XPCShmem {
    fn from(value: &XPCObject) -> Self {
        unsafe {
            xpc_retain(value.as_ptr());
        }
        Self::from_xpc_object(value.clone())
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

        let ok = unsafe {
            mach_vm_deallocate(
                task.unwrap_or(mach_task_self()),
                *region as mach_vm_address_t,
                *size,
            )
        };

        if ok != 0 {
            panic!("shmem won't drop (vm_deallocate errno {})", ok);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::dict_builder::DictBuilder;
    use crate::object::xpc_object::{XPCHashMap, XPCObject};
    use crate::object::xpc_shmem::XPCShmem;
    use crate::{
        dispatch_get_global_queue, xpc_connection_create, xpc_connection_create_from_endpoint,
        xpc_connection_resume, xpc_connection_send_message, xpc_connection_set_event_handler,
        xpc_connection_t, xpc_endpoint_create, xpc_object_t, xpc_retain, xpc_shmem_map,
        DISPATCH_QUEUE_PRIORITY_HIGH,
    };
    use block::ConcreteBlock;
    use libc::MAP_SHARED;
    use std::collections::HashMap;

    use crate::object::try_xpc_into_rust::TryXPCIntoRust;
    use std::ffi::c_void;
    use std::ops::Deref;
    use std::ptr::{null, null_mut};
    use std::slice::from_raw_parts;
    use std::sync::mpsc;

    fn activate_with_handler<F>(peer: xpc_connection_t, f: F) -> xpc_connection_t
    where
        F: Fn(xpc_object_t) + 'static,
    {
        let block = ConcreteBlock::new(f);
        let block = block.copy();
        unsafe {
            xpc_connection_set_event_handler(peer, &*block as *const _ as *mut _);
            xpc_connection_resume(peer);
        }

        peer
    }

    #[test]
    fn shmem_self_trip() {
        // Make a shmem with 0-127, check that it's there
        let nums: Vec<u8> = (0..128).collect();

        // Pages aligned by 16k on aarch64 and 4k on amd64. If you pass a smaller number
        // you will get the minimum page size per alignment
        let shmem = XPCShmem::allocate_task_self(16384, MAP_SHARED).expect("Must make shmem");

        let shmem_slice = unsafe {
            shmem
                .region
                .copy_from(nums.as_ptr() as *const c_void, nums.len());
            from_raw_parts(shmem.region as *const u8, nums.len())
        };

        assert!(shmem_slice.to_vec().eq(&nums));

        // Make an anonymous XPC listener
        let queue = unsafe { dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_HIGH as isize, 0) };

        let peer = unsafe { xpc_connection_create(null(), queue) };
        let (tx, rx) = mpsc::channel::<XPCObject>();

        // The outer handler receives an XPC connection, which when activated has the sent message
        activate_with_handler(peer, move |object: xpc_object_t| {
            let closure_tx = tx.clone();

            activate_with_handler(
                object as xpc_connection_t,
                move |recv: xpc_object_t| unsafe {
                    let rx_xpc = XPCObject::from_raw(xpc_retain(recv));
                    closure_tx.send(rx_xpc).expect("Must send")
                },
            );
        });

        let endpoint = unsafe { xpc_endpoint_create(peer) };
        let endpoint_peer = unsafe { xpc_connection_create_from_endpoint(endpoint) };

        // ??? cannot call xpc_connection_resume/activate without a handler bound
        activate_with_handler(endpoint_peer, move |object: xpc_object_t| {
            log::info!("Endpoint peer RX: {:?}", object);
        });

        // Send XPC dictionary with a shmem field
        let dict: XPCObject = HashMap::new().entry("shmem", &shmem.xpc_object).into();

        unsafe {
            xpc_connection_send_message(endpoint_peer, xpc_retain(dict.as_ptr()));
        }

        // Read the same message back from the channel and assert the contents of the shmem
        let recv: XPCHashMap = rx.recv().expect("Must recv").to_rust().expect("Must dict");

        let rx_shmem: XPCShmem = recv.get("shmem").unwrap().deref().into();

        // Map region from handle
        let mut rx_shmem_region: *mut c_void = null_mut();
        unsafe {
            xpc_shmem_map(rx_shmem.xpc_object.as_ptr(), &mut rx_shmem_region);
        }

        let rx_shmem_slice: &[u8] =
            unsafe { from_raw_parts(rx_shmem_region as *const _, nums.len()) };

        assert_eq!(rx_shmem_slice.to_vec(), nums);
    }
}
