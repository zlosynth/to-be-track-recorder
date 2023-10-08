pub use stm32h7xx_hal as hal;

use daisy::led::LedUser;
use hal::pac::CorePeripherals;
use hal::pac::Peripherals as DevicePeripherals;
use hal::time::Hertz;
use systick_monotonic::Systick;

pub struct System {
    pub mono: Systick<1000>,
    pub status_led: LedUser,
    pub system_clock: Hertz,
}

impl System {
    /// Initialize system abstraction.
    ///
    /// # Panics
    ///
    /// The system can be initialized only once. It panics otherwise.
    #[must_use]
    pub fn init(mut cp: CorePeripherals, dp: DevicePeripherals) -> Self {
        enable_cache(&mut cp);

        let board = daisy::Board::take().unwrap();
        let ccdr = daisy::board_freeze_clocks!(board, dp);
        let pins = daisy::board_split_gpios!(board, ccdr, dp);

        let mono = Systick::new(cp.SYST, 480_000_000);
        let status_led = daisy::board_split_leds!(pins).USER;
        let system_clock = ccdr.clocks.sys_ck();

        Self {
            mono,
            status_led,
            system_clock,
        }
    }
}

/// AN5212: Improve application performance when fetching instruction and
/// data, from both internal andexternal memories.
fn enable_cache(cp: &mut CorePeripherals) {
    cp.SCB.enable_icache();
    // NOTE: This requires cache management around all use of DMA.
    cp.SCB.enable_dcache(&mut cp.CPUID);
}
