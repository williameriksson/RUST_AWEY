#![no_std]
//#![deny(unsafe_code)]
#![feature(proc_macro)]

//extern crate cortex_m_semihosting;
extern crate stm32f103xx;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m;

//use cortex_m::{iprint, iprintln};
//use core::fmt::Write;
//use cortex_m::peripheral::SystClkSource;
// use cortex_m_semihosting::hio;
//use stm32f103xx::DWT;
use rtfm::{app, Threshold, Resource};

//const LOGGING_FREQUENCY: u32 = 1; // Hz
//const BAUD_RATE: u32 = 115_200;

app! {
    device: stm32f103xx,

    resources: {
    //    static RECEIVE_BUFFER: [u8; 128] = [0; 128];
    //    static BUFFER_INDEX: u16 = 0;
        static U_SEC_PER_REV: u32 = 0;
        static LAST_TIME_STAMP: u32 = 0;
        static START_POSITION: bool = false;
        static NEXT_SLOT: bool = false;
    },

    idle: {
        resources: [DWT, GPIOA, START_POSITION, NEXT_SLOT],
    },

    tasks: {
        /*SYS_TICK: {
            path: logging,
        },*/

        TIM2: {
            path: blink_led,
            resources: [TIM2, GPIOC, GPIOA, START_POSITION, NEXT_SLOT],
        },

       /*USART1: {
            path: cmd,
            resources: [USART1, RECEIVE_BUFFER, BUFFER_INDEX, TIM2] 
       },*/

       EXTI1: {
            path: hall_sensor,
            resources: [EXTI, START_POSITION, DWT, U_SEC_PER_REV, LAST_TIME_STAMP, NEXT_SLOT, TIM2],
       },
    }, 
} 


fn init(p: ::init::Peripherals, _r: ::init::Resources) { 
    
    
    p.RCC.apb2enr.write(|w| w.iopcen().set_bit()); // Enable GPIOC clock 
    
    
    // Configure the SYSTICK
    /*p.SYST.set_clock_source(SystClkSource::Core);
    p.SYST.set_reload(8_000_000 / LOGGING_FREQUENCY);
    p.SYST.enable_interrupt();
    p.SYST.enable_counter();*/

    p.GPIOC.crh.write(|w| w.mode13().bits(1));          // Set PC13 as outpus (max 2 MHz) (user LED on bluepull)

    p.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());  // Enable TIM2 peripheral clock
   
    let prescaler = 80 - 1;                           // 8 Mhz / 80 = 100kHz
    p.TIM2.psc.write(|w| w.psc().bits(prescaler));

    
    let arr_value = 50;
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
    

    //p.RCC.apb2enr.modify(|_, w| w.usart1en().set_bit());            // Enable USART1 clock
    p.RCC.apb2enr.modify(|_, w| w.iopaen().set_bit());              // Enable GPIOA clock
    //p.GPIOA.crh.modify(|_, w| w.mode9().bits(1).cnf9().bits(2));    // Set PA9 as output and then AF Push-pull (FOR USART)


    /*p.USART1.cr1.write(|w| w.ue().set_bit().re().set_bit().te().set_bit().rxneie().set_bit());
    let baud = 8_000_000 / BAUD_RATE;
   
    p.USART1.brr.write(|w| unsafe{w.bits(baud)});*/



    p.GPIOA.crl.modify(|_, w| w.mode7().bits(3).cnf7().bits(0));    // Set PA7 as output
    p.GPIOA.crl.modify(|_, w| w.mode6().bits(3).cnf6().bits(0));    // Set PA6 as output

    p.GPIOA.crl.modify(|_, w| w.mode5().bits(3).cnf5().bits(0));    // Set PA5 as output
    p.GPIOA.crl.modify(|_, w| w.mode4().bits(3).cnf4().bits(0));    // Set PA4 as output
    p.GPIOA.crl.modify(|_, w| w.mode3().bits(3).cnf3().bits(0));    // Set PA3 as output
    p.GPIOA.crl.modify(|_, w| w.mode2().bits(3).cnf2().bits(0));    // Set PA2 as output

    p.GPIOA.odr.write(|w| w.odr2().bit(false));
    p.GPIOA.odr.write(|w| w.odr3().bit(false));
    p.GPIOA.odr.write(|w| w.odr4().bit(false));
    p.GPIOA.odr.write(|w| w.odr5().bit(false));
    p.GPIOA.odr.write(|w| w.odr6().bit(false));
    p.GPIOA.odr.write(|w| w.odr7().bit(false));


    p.AFIO.exticr1.modify(|_, w| unsafe{w.exti1().bits(0)});
    p.EXTI.imr.modify(|_, w| w.mr1().set_bit());
    p.EXTI.ftsr.modify(|_, w| w.tr1().set_bit());

    //iprintln!(&p.ITM.stim[0], "init done");
}

fn idle(t: &mut Threshold, mut r: ::idle::Resources) -> ! {
 
    let T = [
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, true],
        [true, false, false, false, false],
        [true, false, false, false, false]];

   
    let E = [
        [true, true, true, true, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true]];

     
    let S = [
        [true, true, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, true, true]];

    let text = [
    
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, true],
        [true, false, false, false, false],
        [true, false, false, false, false],

        [false, false, false, false, false],

        [true, true, true, true, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, false, false, true],
        [true, false, false, false, true],


        [false, false, false, false, false],

        [true, true, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, false, true],
        [true, false, true, true, true],


        [false, false, false, false, false],
        
        [true, false, false, false, false],
        [true, false, false, false, false],
        [true, true, true, true, true],
        [true, false, false, false, false],
        [true, false, false, false, false],

        [false, false, false, false, false] ];
    /*
    let text = [[true, false, false, false, false], 
                [true, true, false, false, false], 
                [true, true, true, false, false], 
                [true, true, true, true, false], 
                [true, true, true, true, true], 
                [false, false, false, false, false] ];*/
    loop {
        let mut start_position = false;
        r.START_POSITION.claim(t, |v, _t| start_position = **v);
        
        
        if start_position {
           
            for column in text.iter() {
            

                let mut next_slot = false;
                while next_slot == false {
                    r.NEXT_SLOT.claim(t, |v, _t| next_slot = **v);
                }

                r.GPIOA.claim_mut(t, |gpioa, _t| { 
                    gpioa.odr.write(|w| w.
                        odr7().bit(column[0]).
                        odr6().bit(column[1]).
                        odr5().bit(column[2]).
                        odr4().bit(column[3]).
                        odr3().bit(column[4]).
                        odr2().bit(true));
                });

            
                r.NEXT_SLOT.claim_mut(t, |next_slot, _t| { 
                    **next_slot = false;
                });

            }



            r.START_POSITION.claim_mut(t, |start_pos, _t| { 
                **start_pos = false;
            });
            //r.START_POSITION = false;
        }

    }
}

fn hall_sensor(t: &mut Threshold, r: EXTI1::Resources) {
   
    let current_time_stamp = r.DWT.cyccnt.read();
    let u_sec_per_rev = current_time_stamp - **r.LAST_TIME_STAMP;

    **r.U_SEC_PER_REV = u_sec_per_rev;
    **r.LAST_TIME_STAMP = current_time_stamp;
    **r.START_POSITION = true;
    **r.NEXT_SLOT = true;


    //let arr_value = 50;
    //let arr_value = u_sec_per_rev / 40000;
    //r.TIM2.arr.write(|w| unsafe{w.bits(arr_value)});    // Auto reload register
    
    r.EXTI.claim_mut(t, |exti, _t| {
        exti.pr.write(|w| w.pr1().set_bit()); 
    
    });

}

/*fn logging(t: &mut Threshold, r: SYS_TICK::Resources) {
    
    /*let mut percent: u32 = 0;

    let mut time_idle = 0;
    let mut time_busy = 0;

    r.IDLE_COUNTER.claim(t, |v, _t| time_idle = **v);
    r.BUSY_COUNTER.claim(t, |v, _t| time_busy = **v);

    let total_time = time_idle.wrapping_add(time_busy); 
    if total_time > 0 {
        percent = time_busy.wrapping_mul(100).wrapping_div(total_time);
    }*/
 

    //if percent < 50 {
    //    for _ in 0..8000 {};
    //} else {
    //    for _ in 0..3000 {};
   // }
    
    //iprintln!(&r.ITM.stim[0],"busy: {}, idle: {}, percent: {}", time_busy, time_idle, percent);
}*/

fn blink_led(t: &mut Threshold, r: TIM2::Resources) {
    

   /* let mut blink_enable = true;
    r.BLINK_ENABLE.claim(t, |v, _t| blink_enable = **v);

    if blink_enable == false {
        return;
    }*/

    r.TIM2.claim_mut(t, |tim2, _t| {
        tim2.sr.write(|w| w.uif().clear_bit()); 
    
    });
    **r.NEXT_SLOT = true;


    /*r.GPIOA.claim_mut(t, |gpioa, _t| { 
        gpioa.odr.write(|w| w.
            odr7().bit(true).
            odr6().bit(true).
            odr5().bit(true).
            odr4().bit(true).
            odr3().bit(true).
            odr2().bit(true));
    });*/
    

    /*if **r.START_POSITION == true {
        r.GPIOC.claim_mut(t, |gpioc, _t| {
        
            gpioc.odr.write(|w| w.odr13().bit( !gpioc.odr.read().odr13().bit() ));
        });
    }
    **r.START_POSITION = false;*/
}

/*
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


     
}*/



