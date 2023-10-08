#![no_main]
#![no_std]
#![allow(clippy::no_mangle_with_rust_abi)] // rtic::app fails this.

use placeholder_firmware as _; // Global logger and panicking behavior.

#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true, dispatchers = [EXTI0, EXTI1, EXTI2])]
mod app {
    use daisy::led::LedUser;
    use fugit::ExtU64;
    use systick_monotonic::Systick;

    use placeholder_firmware::system::System;

    // Blinks on the PCB's LED signalize the revision.
    const BLINKS: u8 = 1;

    // 1 kHz / 1 ms granularity for task scheduling.
    #[monotonic(binds = SysTick, default = true)]
    type Mono = Systick<1000>;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        status_led: LedUser,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("Starting the firmware, initializing resources");

        let system = System::init(cx.core, cx.device);
        let mono = system.mono;
        let status_led = system.status_led;

        blink::spawn(true, BLINKS).unwrap();

        (Shared {}, Local { status_led }, init::Monotonics(mono))
    }

    #[task(local = [status_led])]
    fn blink(cx: blink::Context, on: bool, mut blinks_left: u8) {
        let status_led = cx.local.status_led;

        let time_on = 200.millis();
        let time_off_short = 200.millis();
        let time_off_long = 2.secs();

        if on {
            status_led.set_high();
            blink::spawn_after(time_on, false, blinks_left).unwrap();
        } else {
            status_led.set_low();
            blinks_left -= 1;
            if blinks_left > 0 {
                blink::spawn_after(time_off_short, true, blinks_left).unwrap();
            } else {
                blink::spawn_after(time_off_long, true, BLINKS).unwrap();
            }
        }
    }
}
