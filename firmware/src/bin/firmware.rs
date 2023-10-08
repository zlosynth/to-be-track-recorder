#![no_main]
#![no_std]
#![allow(clippy::no_mangle_with_rust_abi)] // rtic::app fails this.

use placeholder_firmware as _; // Global logger and panicking behavior.

#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use systick_monotonic::Systick;

    // 1 kHz / 1 ms granularity for task scheduling.
    #[monotonic(binds = SysTick, default = true)]
    type Mono = Systick<1000>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("Starting the firmware, initializing resources");

        let systick = cx.core.SYST;
        let mono = Systick::new(systick, 480_000_000);

        (Shared {}, Local {}, init::Monotonics(mono))
    }
}
