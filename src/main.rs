#![no_main]
#![no_std]

use panic_semihosting as _;

#[rtic::app(device = stm32f3xx_hal::stm32, peripherals = true)]
mod app {
    // use cortex_m_semihosting::{debug, hprintln};
    use stm32f3xx_hal::delay::Delay;
    use stm32f3xx_hal::pac::{self, DAC};
    use stm32f3xx_hal::prelude::*;

    #[resources]
    struct Resources {
        dac: DAC,
        delay: Delay,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Cortex-M peripherals
        let _core: cortex_m::Peripherals = cx.core;

        // Device specific peripherals
        let dp: pac::Peripherals = cx.device;

        dp.RCC.apb1enr.write(|w| w.dac1en().enabled());
        let mut rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.freeze(&mut dp.FLASH.constrain().acr);

        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
        let _pa4 = gpioa.pa4.into_analog(&mut gpioa.moder, &mut gpioa.pupdr);

        dp.DAC.cr.write(|w| {
            w.boff1().enabled();
            w.en1().enabled()
        });
        dp.DAC.dhr12r1.write(|w| unsafe { w.bits(0x7FF) });

        init::LateResources {
            dac: dp.DAC,
            delay: Delay::new(_core.SYST, clocks),
        }
    }

    #[idle(resources = [dac, delay])]
    fn idle(mut cx: idle::Context) -> ! {
        let mut dac_val: u32 = 0;
        loop {
            cx.resources.dac.lock(|dac| {
                dac.dhr12r1.write(|w| unsafe { w.bits(dac_val) });
            });
            dac_val = (dac_val + 1) % 0x1000;
            cx.resources.delay.lock(|delay| {
                const PERIOD: u32 = (1_000_000 / 4096) * 5;
                delay.delay_us(PERIOD);
            });
        }
    }
}
