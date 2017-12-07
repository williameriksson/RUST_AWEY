#![no_std]
//#![deny(unsafe_code)]
#![feature(proc_macro)]


extern crate cortex_m_semihosting;
extern crate stm32f103xx;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m;

use core::fmt::Write;
use cortex_m::peripheral::SystClkSource;
use cortex_m_semihosting::hio;
use stm32f103xx::DWT;
use rtfm::{app, Threshold, Resource};

const FREQUENCY: u32 = 1; // Hz

app! {
    device: stm32f103xx,

    resources: {
        static IDLE_COUNTER: u64 = 0;
        static BUSY_COUNTER: u64 = 0;
       // static BLINK_PERIOD: u16 = 100;
       // static CPU_USAGE: u8 = 0;
    },

    idle: {
        resources: [IDLE_COUNTER, BUSY_COUNTER, GPIOC],
    },

    // Both SYS_TICK and TIM2 have access to the `COUNTER` data
    tasks: {
        SYS_TICK: {
            path: logging,
            resources: [DWT, IDLE_COUNTER, BUSY_COUNTER],
        },

        TIM2: {
            path: blink_led,
            resources: [TIM2, GPIOC],
        },

       // BLINK: {
       //     path: blink,
       //     resources: [BLINK_PERIOD],
       // },

       // USART1: {
       //     path: usart2,
       //     resources: [USART1]
    
       // }
    },
}

fn init(p: ::init::Peripherals, _r: ::init::Resources) {
    
    p.RCC.apb2enr.write(|w| w.iopcen().set_bit()); // Enable GPIOC clock
   
    // Configure the SYSTICK
    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000 / FREQUENCY);
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();

    // PC13 is wired to the bluepill user LED
    p.GPIOC.crh.write(|w| w.mode13().bits(1)); // Set PC13 as outpus (max 2 MHz)
    //p.GPIOC.odr.write(|w| w.odr13().bit(true));

    p.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit()); // Enable TIM2 peripheral clock
   
    let prescaler = 16000 - 1; // 16 Mhz / 16000 = 1kHz
    p.TIM2.psc.write(|w| unsafe{w.psc().bits(prescaler)});

    // Set the auto reload register
    let arr_value = 1000;
    p.TIM2.arr.write(|w| unsafe{w.bits(arr_value)});

    p.TIM2.dier.write(|w| w.uie().set_bit());   // Interrupt enable TIM2 update event
    p.TIM2.egr.write(|w| w.ug().set_bit());     // Reset on timer interrupt
    p.TIM2.cr1.write(|w| w.arpe().set_bit());   // Auto reload on interrupt
    p.TIM2.cr1.write(|w| w.cen().bit(true));    // Enable TIM2

    unsafe {
        p.DWT.cyccnt.write(0);
        p.DWT.sleepcnt.write(0);
    }
    p.DWT.enable_cycle_counter();
}

fn idle(t: &mut Threshold, mut r: ::idle::Resources) -> ! {
    let mut stdout = hio::hstdout().unwrap();
  
   // r.GPIOC.odr.write(|w| w.odr13().bit(false));
    loop {
        r.IDLE_COUNTER.claim_mut(t, |v, _t| **v = 0);

        rtfm::wfi();
        //let led_on = r.GPIOC.odr.read().odr13().bit();
        //writeln!(stdout, "{}", led_on);
        //r.GPIOC.odr.write(|w| w.odr13().bit(!led_on));
       
        
        //r.COUNTER.claim_mut(t, |v, _t| **v = 0); 
    }
}

fn logging(t: &mut Threshold, r: SYS_TICK::Resources) {
    
    let mut stdout = hio::hstdout().unwrap();
    let cyc = r.DWT.cyccnt.read();
    let slp = r.DWT.sleepcnt.read();
    // writeln!(stdout, "cyc: {}", cyc);
    // writeln!(stdout, "slp: {}", slp);
    //r.COUNTER.claim_mut(t, |v, _t| **v = 0); 
}

fn blink_led(t: &mut Threshold, r: TIM2::Resources) {
     
    //let mut led_on = false;
   
    r.TIM2.claim_mut(t, |tim2, _t| {
        tim2.sr.write(|w| w.uif().clear_bit()); 
    
    });

    r.GPIOC.claim_mut(t, |gpioc, _t| {
        
        gpioc.odr.write(|w| w.odr13().bit( !gpioc.odr.read().odr13().bit() ));
    });
    //r.COUNTER.claim_mut(t, |v, _t| **v = 0); 
}
//fn main() {

    // (*DWT.get()).enable_cycle_counter();
   // (*DWT.get()).cyccnt.write(0);
   // cycle_count = (*DWT.get()).cyccnt.read();
    
//}
