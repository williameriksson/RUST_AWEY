#![no_std]
//#![deny(unsafe_code)]
#![feature(proc_macro)]

//extern crate cortex_m_semihosting;
extern crate stm32f103xx;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m;

use cortex_m::{iprint, iprintln};
//use core::fmt::Write;
use cortex_m::peripheral::SystClkSource;
// use cortex_m_semihosting::hio;
//use stm32f103xx::DWT;
use rtfm::{app, Threshold, Resource};

const FREQUENCY: u32 = 1; // Hz

app! {
    device: stm32f103xx,

    resources: {
        static IDLE_COUNTER: u32 = 0;
        static BUSY_COUNTER: u32 = 0;
       // static BLINK_PERIOD: u16 = 100;
    },

    idle: {
        resources: [IDLE_COUNTER, BUSY_COUNTER, GPIOC, ITM, DWT],
    },

    tasks: {
        SYS_TICK: {
            path: logging,
            resources: [DWT, IDLE_COUNTER, BUSY_COUNTER, ITM],
        },

        TIM2: {
            path: blink_led,
            resources: [TIM2, GPIOC],
        },

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
   
    let prescaler = 8000 - 1; // 8 Mhz / 8000 = 1kHz
    p.TIM2.psc.write(|w| w.psc().bits(prescaler));

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

    iprintln!(&p.ITM.stim[0], "init done");
}

fn idle(t: &mut Threshold, mut r: ::idle::Resources) -> ! {
 
    let mut prev_idle_end = 0;
    loop {
        cortex_m::interrupt::disable();
        
        let mut idle_start = 0;
        r.DWT.claim(t, |v, _t| idle_start = v.cyccnt.read());
        
        rtfm::wfi();
         
        let mut idle_end = 0;
        r.DWT.claim(t, |v, _t| idle_end = v.cyccnt.read());

        let time_idle = idle_end.wrapping_sub(idle_start);
        let time_busy = idle_start.wrapping_sub(prev_idle_end);

        r.IDLE_COUNTER.claim_mut(t, |v, _t| **v = time_idle); 
        r.BUSY_COUNTER.claim_mut(t, |v, _t| **v = time_busy);
        
        prev_idle_end = idle_end;

        unsafe{cortex_m::interrupt::enable();}

    }
}

fn logging(t: &mut Threshold, r: SYS_TICK::Resources) {
    
    let mut percent: u32 = 0;

    let mut time_idle = 0;
    let mut time_busy = 0;

    r.IDLE_COUNTER.claim(t, |v, _t| time_idle = **v);
    r.BUSY_COUNTER.claim(t, |v, _t| time_busy = **v);

    let total_time = time_idle.wrapping_add(time_busy); 
    if total_time > 0 {
        percent = time_busy.wrapping_mul(100).wrapping_div(total_time);
    }
  
    if percent < 50 {
        for _ in 0..8000 {};
    } else {
        for _ in 0..3000 {};
    }
    
    iprintln!(&r.ITM.stim[0],"busy: {}, idle: {}, percent: {}", time_busy, time_idle, percent);
}

fn blink_led(t: &mut Threshold, r: TIM2::Resources) {
     
    r.TIM2.claim_mut(t, |tim2, _t| {
        tim2.sr.write(|w| w.uif().clear_bit()); 
    
    });

    r.GPIOC.claim_mut(t, |gpioc, _t| {
        
        gpioc.odr.write(|w| w.odr13().bit( !gpioc.odr.read().odr13().bit() ));
    });
}
