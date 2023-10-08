#![no_main]
#![no_std]

use defmt_rtt as _; // Global logger.
use panic_probe as _;
use stm32h7xx_hal as _; // Readable panic.

// Same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked.
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}
