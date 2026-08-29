#![cfg_attr(target_arch = "wasm32", no_std)]
pub mod abi;
pub mod kernel;
#[cfg(test)]
mod offline;
pub mod spsc;

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
