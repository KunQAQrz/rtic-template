#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

use panic_rtt_target as _;
use rtic_monotonics::systick::prelude::*;
use rtt_target::{rprintln, rtt_init_print};
use stm32h7xx_hal::prelude::*;

systick_monotonic!(Mono, 1000);

#[rtic::app(device = stm32h7xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use super::*;
    use stm32h7xx_hal::gpio;
    use stm32h7xx_hal::gpio::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: gpio::Pin<'C', 1, Output<PushPull>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        rtt_init_print!();
        rprintln!("init start");

        rprintln!("Setup clocks");
        let rcc = cx.device.RCC.constrain();

        let pwr = cx.device.PWR.constrain();
        let pwr_cfg = pwr.freeze();

        // Initialize the systick interrupt & obtain the token to prove that we did
        Mono::start(cx.core.SYST, 200_000_000); // default STM32H743 clock-rate is 200MHz

        let ccdr = rcc.sys_ck(200.MHz()).freeze(pwr_cfg, &cx.device.SYSCFG);

        rprintln!("Setup LED");
        let gpio: gpioc::Parts = cx.device.GPIOC.split(ccdr.peripheral.GPIOC);
        let mut led = gpio.pc1.into_push_pull_output();
        led.set_high();

        // Schedule the blinking task
        blink::spawn().ok();

        rprintln!("init end");

        (Shared {}, Local { led })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        loop {
            cx.local.led.toggle();

            if let PinState::High = cx.local.led.get_state() {
                rprintln!("LED off");
            } else {
                rprintln!("LED on");
            }

            Mono::delay(1000.millis()).await;
        }
    }
}
