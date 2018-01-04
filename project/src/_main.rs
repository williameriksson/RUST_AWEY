#![no_std]
#![deny(unsafe_code)]
#![feature(proc_macro)]


extern crate cortex_m_semihosting;
extern crate stm32f103xx;
extern crate cortex_m_rtfm as rtfm;

use core::fmt::Write;

use cortex_m_semihosting::hio;
//use stm32f103xx::DWT;
use rtfm::{app, Threshold};

app! {
    device: stm32f103xx,

   // resources: {
       // static COUNTER: u64 = 0;
       // static BLINK_PERIOD: u16 = 100;
       // static CPU_USAGE: u8 = 0;
   // },

    // Both SYS_TICK and TIM2 have access to the `COUNTER` data
    tasks: {
        SYS_TICK: {
            path: sys_tick,
            resources: [COUNTER],
        },

        TIM2: {
            path: tim2,
            resources: [TIM2, COUNTER],
        },

        BLINK: {
            path: blink,
            resources: [BLINK_PERIOD],
        },

        USART1: {
            path: usart2,
            resources: [USART1]

        }
    },
}

fn init(p: ::init::Peripherals, _r: ::init::Resources) {
    
    p.rcc.apb2enr.write(|w| w.gpiocen().set_bit());

}


//fn main() {

    // (*DWT.get()).enable_cycle_counter();
   // (*DWT.get()).cyccnt.write(0);
   // cycle_count = (*DWT.get()).cyccnt.read();
    
//}
