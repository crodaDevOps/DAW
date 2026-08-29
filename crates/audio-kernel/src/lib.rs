#![cfg_attr(target_arch = "wasm32", no_std)]
#[cfg(target_arch = "wasm32")]
extern crate alloc;
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
pub mod abi;
pub mod kernel;
#[cfg(test)]
mod offline;
pub mod spsc;
pub mod wasm_abi;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
