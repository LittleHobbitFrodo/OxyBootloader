pub mod int {
    use core::arch::asm;
    
    #[inline(always)]
    pub fn disable() {
        unsafe { asm!("cli"); }
    }

    #[inline(always)]
    pub fn enable() {
        unsafe { asm!("sti"); }
    }
}
pub fn hang() -> ! {
    loop {
        int::disable();
        unsafe { core::arch::asm!("hlt"); }
        core::hint::spin_loop();
    }
}