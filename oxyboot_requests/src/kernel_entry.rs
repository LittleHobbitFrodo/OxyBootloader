use crate::{Id, NoResponse, Request, RequestBase, Response};


/// Tells the bootloader which function should it all upon boot
/// - this request has no response
#[repr(C, align(8))]
pub struct KernelEntryRequest {
    base: RequestBase<NoResponse>,
    entry: fn(),
}

unsafe impl Sync for KernelEntryRequest {}
unsafe impl Send for KernelEntryRequest {}

impl KernelEntryRequest {

    /// identifier of this entry
    /// - "OXYBOOT BOOT REQUEST KERN ENTRY" in ascii (append one zero)
    pub const ID: Id = [79, 88, 89, 66, 79, 79, 84, 32, 66, 79, 79,
        84, 32, 82, 69, 81, 85, 69, 83, 84, 32, 75, 69, 82, 78, 32,
        69, 78, 84, 82, 89, 0];

    #[cfg(feature = "bootloader")]
    pub fn build_new(revision: u64, entry: fn()) -> Self {
        Self {
            base: RequestBase { id: Self::ID, revision, response: None },
            entry: entry
        }
    }

    #[cfg(feature = "kernel")]
    pub const fn new(revision: u64, entry: fn()) -> Self {
        Self {
            base: RequestBase::new(Self::ID, revision),
            entry: unsafe { core::mem::transmute(entry) },
        }
    }

    #[inline(always)]
    pub fn entry(&self) -> fn() {
        unsafe { core::mem::transmute(self.entry) }
    }


}

impl Request<NoResponse> for KernelEntryRequest {
    fn revision(&self) -> crate::Revision { self.base.revision }
    fn response(&self) -> Option<&'static NoResponse> { None }
    fn response_mut(&self) -> Option<&'static mut NoResponse> { None }

    #[cfg(feature = "bootloader")]
    #[inline]
    fn is_valid(ptr: *const Self) -> bool {
        unsafe { ptr.as_ref().unwrap() }.base.id == Self::ID
    }
}