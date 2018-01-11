#![no_std]
//#![deny(unsafe_code)]
#![feature(proc_macro)]

//extern crate cortex_m_semihosting;
extern crate stm32f103xx;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m;

use cortex_m::{iprint, iprintln};
use cortex_m::peripheral::SystClkSource;
use rtfm::{app, Threshold, Resource};

const LOGGING_FREQUENCY: u32 = 1; // Hz
const BAUD_RATE: u32 = 115_200;


app! {
    device: stm32f103xx,

    resources: {
        static IDLE_COUNTER: u32 = 0;
        static BUSY_COUNTER: u32 = 0;
        static RECEIVE_BUFFER: [u8; 128] = [0; 128];
        static BUFFER_INDEX: u16 = 0;
        static BLINK_ENABLE: bool = false;
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
            resources: [TIM2, GPIOC, BLINK_ENABLE],
        },

       USART1: {
            path: cmd,
            resources: [USART1, RECEIVE_BUFFER, BUFFER_INDEX, BLINK_ENABLE, TIM2] 
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
 
    let mut prev_time = 0;
    loop {
        cortex_m::interrupt::disable();
        
        // Read current clockcycle count
        let mut current_count = 0;
        r.DWT.claim(t, |v, _t| current_count = v.cyccnt.read());
        // Increment the busy counter with the number of clockcycles since last idle ended until now when the last work ended.
        r.BUSY_COUNTER.claim_mut(t, |v, _t| **v = (**v).wrapping_add(current_count.wrapping_sub(prev_time)));

        // Read the counter just before the sleep begins.
        let mut before_sleep = 0;
        r.DWT.claim(t, |v, _t| before_sleep= v.cyccnt.read());
        
        rtfm::wfi();
        
        // Interrupt arrived, read the clock cycle count after sleep and
        // increment the idle counter with the sleep duration.
        r.DWT.claim(t, |v, _t| current_count = v.cyccnt.read());
        r.IDLE_COUNTER.claim_mut(t, |v, _t| **v = (**v).wrapping_add(current_count.wrapping_sub(before_sleep))); 

        // Save the cycle count for the end of current idle / start of the new work.
        prev_time = current_count;
        
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
 

    iprintln!(&r.ITM.stim[0],"busy: {}, idle: {}, percent: {}", time_busy, time_idle, percent);

    // Reset the idle and the busy counters
    r.IDLE_COUNTER.claim_mut(t, |v, _t| **v = 0);
    r.BUSY_COUNTER.claim_mut(t, |v, _t| **v = 0);
}

fn blink_led(t: &mut Threshold, r: TIM2::Resources) {
    
    // Check if in paused state or not
    let mut blink_enable = true;
    r.BLINK_ENABLE.claim(t, |v, _t| blink_enable = **v);

    // If in paused state, return
    if blink_enable == false {
        return;
    }

    // Reset interrupt flag
    r.TIM2.claim_mut(t, |tim2, _t| {
        tim2.sr.write(|w| w.uif().clear_bit()); 
    
    });

    // Toggle the LED 
    r.GPIOC.claim_mut(t, |gpioc, _t| {
        gpioc.odr.write(|w| w.odr13().bit( !gpioc.odr.read().odr13().bit() ));
    });
}


#[allow(non_snake_case)]
fn usart_tx<A>(t: &mut Threshold, usart: &mut A, tx_str: &str) where A: Resource<Data = stm32f103xx::USART1>, {

    
    usart.claim_mut(t, |usart1, _t| {
        for b in tx_str.chars() {
            usart1.dr.write(|w| unsafe{w.dr().bits(b as u16)});
            while usart1.sr.read().tc().bit_is_clear() {}
        }
    });
}


#[allow(non_snake_case)]
fn cmd(t: &mut Threshold, USART1::Resources {USART1, RECEIVE_BUFFER, BUFFER_INDEX, BLINK_ENABLE, TIM2}:USART1::Resources ) {
   
    let b = USART1.dr.read().bits() as u8;
    //iprintln!(&ITM.stim[0],"IN CMD: {}", b);

    if b == 0x0a {
        match core::str::from_utf8(&RECEIVE_BUFFER[..**BUFFER_INDEX as usize]) {
            Ok(c) =>  {
                let mut split_cmd = c.split(' ');
                
                match split_cmd.next().unwrap() {
                    "start" => {
                        **BLINK_ENABLE = true;
                        usart_tx(t, USART1, "Blinking started \n \r")
                    },
                    "pause" => {
                        **BLINK_ENABLE = false;
                        usart_tx(t, USART1, "Blinking paused \n \r")
                    },
                    "period" => {
                        let value = split_cmd.next().unwrap();
        
                        let parsed = value.parse::<u32>();
                            match parsed {
                            
                                Ok(int_val) => {
                                    if int_val > 0 && int_val <= 1000 {

                                        TIM2.arr.write(|w| unsafe{w.bits(int_val)});    // Auto reload register
                                        TIM2.cnt.write(|w| unsafe{w.bits(0)});
                                        usart_tx(t, USART1, "Blinking period changed \n \r");
                                    } else {
                                        usart_tx(t, USART1, "Period must be > 0 and < 1000 ms \n \r")
                                    }
                                },
                                Err(_) => usart_tx(t, USART1, "Error in parsing message \n \r"),
                            }
                             
                    },
                    _ => usart_tx(t, USART1, "Command not recognized \n \r"),

                };

            },
            Err(_) => {
                usart_tx(t, USART1, "Error in parsing message \n \r")
                           
            },
        };

        **BUFFER_INDEX = 0;

    } else {
        RECEIVE_BUFFER[**BUFFER_INDEX as usize] = b;
        **BUFFER_INDEX += 1;
    }


     
}



