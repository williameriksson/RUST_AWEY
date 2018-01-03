#![no_std]
//#![deny(unsafe_code)]
#![feature(proc_macro)]

//extern crate cortex_m_semihosting;
extern crate stm32f103xx;
extern crate cortex_m_rtfm as rtfm;
extern crate cortex_m;


mod characters;
//use cortex_m::{iprint, iprintln};
//use core::fmt::Write;
//use cortex_m::peripheral::SystClkSource;
// use cortex_m_semihosting::hio;
//use stm32f103xx::DWT;
use rtfm::{app, Threshold, Resource};
use characters::characters::*;
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
        resources: [GPIOA, START_POSITION, NEXT_SLOT, TIM2],
    },

    tasks: {
        /*SYS_TICK: {
            path: logging,
        },*/

        TIM2: {
            priority: 1,
            path: blink_led,
            resources: [TIM2, GPIOC, GPIOA, START_POSITION, NEXT_SLOT],
        },

        /*TIM3: {
            priority: 3,
            path: tim5,
            resources: [TIM3, GPIOC],
        },*/

       /*USART1: {
            path: cmd,
            resources: [USART1, RECEIVE_BUFFER, BUFFER_INDEX, TIM2] 
       },*/

       EXTI1: {
            priority: 2,
            path: hall_sensor,
            resources: [EXTI, START_POSITION, U_SEC_PER_REV, LAST_TIME_STAMP, NEXT_SLOT, TIM2, DWT, TIM3, GPIOC],
       },
    }, 
} 


fn init(p: ::init::Peripherals, _r: ::init::Resources) { 
    
    
    p.RCC.apb2enr.write(|w| w.iopcen().set_bit()); // Enable GPIOC clock 
    
    p.GPIOC.crh.write(|w| w.mode13().bits(1));          // Set PC13 as outpus (max 2 MHz) (user LED on bluepull)

    p.RCC.apb1enr.modify(|_, w| w.tim2en().set_bit());  // Enable TIM2 peripheral clock
   
    let prescaler = 80 - 1;                           // 8 Mhz / 80 = 100kHz
    p.TIM2.psc.write(|w| w.psc().bits(prescaler));

    
    let arr_value = 25;
    p.TIM2.arr.write(|w| unsafe{w.bits(arr_value)});    // Auto reload register

    p.TIM2.dier.write(|w| w.uie().set_bit());   // Interrupt enable TIM2 update event
    p.TIM2.egr.write(|w| w.ug().set_bit());     // Reset on timer interrupt
    p.TIM2.cr1.write(|w| w.arpe().set_bit());   // Auto reload on interrupt
    p.TIM2.cr1.write(|w| w.cen().bit(true));    // Enable TIM2

    

    p.RCC.apb1enr.modify(|_, w| w.tim3en().set_bit());  // Enable TIM3 peripheral clock 
    
    let tim5_psc = 8 - 1; // 8Mhz / 8 = 1Mhz
    p.TIM3.psc.write(|w| w.psc().bits(tim5_psc));
    
    let tim5_arr_value = 0xFFFF;
    p.TIM3.arr.write(|w| unsafe{w.bits(tim5_arr_value)});    // Auto reload register 

    //p.TIM3.dier.write(|w| w.uie().set_bit());   // Interrupt enable TIM2 update event
    p.TIM3.egr.write(|w| w.ug().set_bit());     // Reset on timer interrupt
    p.TIM3.cr1.write(|w| w.arpe().set_bit());   // Auto reload on interrupt
    p.TIM3.cr1.write(|w| w.cen().bit(true));    // Enable TIM3


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

}

fn idle(t: &mut Threshold, mut r: ::idle::Resources) -> ! {

    const NR_CHARS: usize = 12; 
    let tmp = [O, P, Q, R, S, T, U, V, W, X, Y, Z];
    let mut text = [[false; 5];  NR_CHARS * 6];

    let mut cnt = 0;
    for c in tmp.iter() {
        
        for column in c.iter() {
            text[cnt] = column.clone();
            cnt += 1;
        }
    }
    


    loop {
        let mut start_position = false;
        r.START_POSITION.claim(t, |v, _t| start_position = **v);
        
        
        if start_position {
           
            r.TIM2.claim_mut(t, |tim2, _| {   
                tim2.cnt.write(|w| unsafe{ w.bits(0)});
            });
            for column in text.iter() {
            

                let mut next_slot = false;
                while next_slot == false {
                    r.NEXT_SLOT.claim(t, |v, _t| next_slot = **v);
                }

                r.NEXT_SLOT.claim_mut(t, |next_slot, _t| { 
                    **next_slot = false;
                });

                r.GPIOA.claim_mut(t, |gpioa, _t| { 
                    gpioa.odr.write(|w| w.
                        odr7().bit(column[0]).
                        odr6().bit(column[1]).
                        odr5().bit(column[2]).
                        odr4().bit(column[3]).
                        odr3().bit(column[4]).
                        odr2().bit(true));
                });

            

            }



            r.START_POSITION.claim_mut(t, |start_pos, _t| { 
                **start_pos = false;
            });
            //r.START_POSITION = false;
        }

    }
}
/*
fn tim5(t: &mut Threshold, mut r: TIM3::Resources) {

 
    r.GPIOC.claim_mut(t, |gpioc, _t| {
        gpioc.odr.write(|w| w.odr13().bit( !gpioc.odr.read().odr13().bit() ));
        
    });

    r.TIM3.claim_mut(t, |tim3, _t| {
        tim3.sr.write(|w| w.uif().clear_bit()); 
    
    });
 
}*/

fn hall_sensor(t: &mut Threshold, mut r: EXTI1::Resources) {
   
    //let current_time_stamp: u32 = r.TIM3.cnt.read().bits() as u32;
    //let current_time_stamp: u32 = 100;

    let mut current_time_stamp: u32 = 1;
    
    r.TIM3.claim(t, |v, _t| current_time_stamp = v.cnt.read().bits() as u32);
        
       // .cnt.read().cnt().bits() as u32;
    
    r.TIM3.claim_mut(t, |tim3, _| {   
        tim3.cnt.write(|w| unsafe{ w.bits(0)});
    });

    //r.TIM3.egr.write(|w| w.tg().set_bit());
    //r.TIM3.cr1.write(|w| w.cen().bit(true));    // Enable TIM3
    /*r.TIM3.cnt.read(|v, _t| {
        current_time_stamp = **v
    });*/
    //let current_time_stamp = r.DWT.cyccnt.read();
    //unsafe {r.DWT.cyccnt.write(0)};
    

    r.START_POSITION.claim_mut(t, |start_pos, _t| {
       **start_pos = true; 
    });


   
    //let u_sec_per_rev = current_time_stamp / 8;
    let u_sec_per_rev = current_time_stamp;
    let u_sec_per_deg = u_sec_per_rev;
    let slot_angle = 6;
    
    //let mut arr_value = u_sec_per_rev * slot_angle / 1_000_000;
    
    let mut arr_value = u_sec_per_deg * slot_angle;
    //let mut arr_value = slot_angle / deg_per_u_sec;

    if arr_value < 10 {
        arr_value = 10;
    } else if arr_value > 0xFFFF {
        arr_value = 0xFFFF;
    }

    arr_value = 25; // !!!! REMEMBER THIS !!!!

    r.TIM2.claim_mut(t, |tim2, _t| {
        tim2.arr.write(|w| unsafe{ w.bits(arr_value)  });   
        tim2.egr.write(|w| w.tg().set_bit());
    });


    //r.TIM2.arr.write(|w| unsafe{w.bits(arr_value)});    // Auto reload register
    //r.TIM2.egr.write(|w| w.tg().set_bit());


    r.EXTI.claim_mut(t, |exti, _t| {
        exti.pr.write(|w| w.pr1().set_bit()); 
    
    });

}

   
fn blink_led(t: &mut Threshold, mut r: TIM2::Resources) {
    

    r.TIM2.claim_mut(t, |tim2, _t| {
        tim2.sr.write(|w| w.uif().clear_bit()); 
    
    });

    r.NEXT_SLOT.claim_mut(t, |next_slot, _t| {
       **next_slot = true; 
    });
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



