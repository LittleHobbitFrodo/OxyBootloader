#![no_std]

use core::ptr::NonNull;

pub mod kernel_entry;

pub use kernel_entry::*;


type Revision = u64;
type Id = [u8; 32];     //  "OXYBOOT BOOT REQUEST XXXXXXXXXXX"


/// This is your regular bootloader request base
/// - it will be filled by the bootloader before the kernel is executed
#[repr(C, align(8))]
pub struct RequestBase<R: Response> {
    id: Id,
    revision: Revision,
    response: Option<NonNull<R>>,
}

unsafe impl<R: Response> Sync for RequestBase<R> {}
unsafe impl<R: Response> Send for RequestBase<R> {}

impl<R: Response> RequestBase<R> {
    
    #[cfg(not(feature = "kernel"))]
    pub const fn new(id: Id, revision: Revision, response: Option<NonNull<R>>) -> Self {
        Self { id, revision, response, }
    }

    #[cfg(feature = "kernel")]
    pub const fn new(id: Id, revision: Revision) -> Self {
        Self { id, revision, response: None }
    }

}

impl<R: Response> Request<R> for RequestBase<R> {

    fn response(&self) -> Option<&'static R> {
        if let Some(res) = self.response {
            Some(unsafe { res.as_ref() })
        } else {
            None
        }
    }

    fn response_mut(&self) -> Option<&'static mut R> {
        if let Some(mut res) = self.response {
            Some(unsafe { res.as_mut() })
        } else {
            None
        }
    }

    fn revision(&self) -> Revision { self.revision }

    #[cfg(feature = "bootloader")]
    //  no `is_valid` function for RequestBase
    fn is_valid(ptr: *const Self) -> bool { false }

}

pub trait Request<R: Response>
where Self: Sync + Send {

    /// Allows you to check the bootloader revision
    fn revision(&self) -> Revision;

    /// Returns reference to the response
    fn response(&self) -> Option<&'static R>;

    /// Returns mutable reference to the response
    fn response_mut(&self) -> Option<&'static mut R>;

    /// Checks if value at given position is the same as ID of this request
    #[cfg(feature = "bootloader")]
    fn is_valid(ptr: *const Self) -> bool;
}

pub trait Response
where Self: Sync + Send {
    fn revision(&self) -> Revision;
}

pub struct NoResponse {}

impl Response for NoResponse {
    fn revision(&self) -> Revision { 0 }
}