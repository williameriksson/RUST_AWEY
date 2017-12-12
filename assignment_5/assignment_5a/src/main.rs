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

const LOGGING_FREQUENCY: u32 = 1; // Hz
const BAUD_RATE: u32 = 115_200;


app! {
    device: stm32f103xx,

    resources: {
        static IDLE_COUNTER: u32 = 0;
        static BUSY_COUNTER: u32 = 0;
       // static BLINK_PERIOD: u16 = 100;
    },

    idle: {
        resources: [IDLE_COUNTER, BUSY_COUNTER, DWT],
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

       USART1: {
            path: cmd,
            resources: [USART1, ITM] 
       }, 
    }, 
} 


fn init(p: ::init::Peripherals, _r: ::init::Resources) { p.RCC.apb2enr.write(|w| w.iopcen().set_bit()); // Enable GPIOC clock 
    // Configure the SYSTICK
    p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000 / LOGGING_FREQUENCY);
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();

    p.RCC.apb2enr.modify(|_, w| w.iopcen().set_bit());  // Enable GPIOC clock
    p.GPIOC.crh.write(|w| w.mode13().bits(1));          // Set PC13 as outpus (max 2 MHz) (user LED on bluepull)

    p.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());  // Enable TIM2 peripheral clock
   
    let prescaler = 8000 - 1;                           // 8 Mhz / 8000 = 1kHz
    p.TIM2.psc.write(|w| w.psc().bits(prescaler));

    
    let arr_value = 1000;
    p.TIM2.arr.write(|w| unsafe{w.bits(arr_value)});    // Auto reload register

    p.TIM2.dier.write(|w| w.uie().set_bit());   // Interrupt enable TIM2 update event
    p.TIM2.egr.write(|w| w.ug().set_bit());     // Reset on timer interrupt
    p.TIM2.cr1.write(|w| w.arpe().set_bit());   // Auto reload on interrupt
    p.TIM2.cr1.write(|w| w.cen().bit(true));    // Enable TIM2

    unsafe {
        p.DWT.cyccnt.write(0);
        p.DWT.sleepcnt.write(0);
    }
    p.DWT.enable_cycle_counter();


    p.RCC.apb2enr.modify(|_, w| w.usart1en().set_bit());            // Enable USART1 clock
    p.RCC.apb2enr.modify(|_, w| w.iopaen().set_bit());              // Enable GPIOA clock
    p.GPIOA.crh.modify(|_, w| w.mode9().bits(1).cnf9().bits(2));    // Set PA9 as output and then AF Push-pull


    p.USART1.cr1.write(|w| w.ue().set_bit().re().set_bit().te().set_bit().rxneie().set_bit());
    let baud = 8_000_000 / BAUD_RATE;
   
    p.USART1.brr.write(|w| unsafe{w.bits(baud)});



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
 

    //if percent < 50 {
    //    for _ in 0..8000 {};
    //} else {
    //    for _ in 0..3000 {};
   // }
    
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

fn cmd(t: &mut Threshold, r: USART1::Resources) {
   
    let b = r.USART1.dr.read().bits() as u8;
    iprintln!(&r.ITM.stim[0],"IN CMD: {}", b);
     
    r.USART1.claim_mut(t, |usart1, _t| {

        usart1.dr.write(|w| unsafe{w.dr().bits(b as u16)});

        //while usart1.sr.read().tc().bit_is_clear() {}
    });
}



