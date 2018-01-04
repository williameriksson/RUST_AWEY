#![no_std]

extern crate cortex_m_semihosting;
extern crate stm32f103xx;


use core::fmt::Write;

use cortex_m_semihosting::hio;
use stm32f103xx::DWT;

fn codgen() -> u32 {
    let mut n: u32 = 0;
    let rotate_amount = 30;
    let mut x;
    let y : i32;
    unsafe {
        x = SEED;
        y = SEED as i32 >> 6;
    }
    // Count the number of 0's in seed
    for k in 0..32 {
        match x >> k & 1 {
            0 => n += 1,
            _ => continue, 
        }
    }
    // Rotate seed left by specified amount
    for _k in 0..rotate_amount {
        let msb = (x & 0x80000000) >> 31;
        x <<= 1;
        match msb {
            1 => x = x | 1,
            _ => continue,
        }
    }
    unsafe {
        SEED = x ^ (y as u32) ^ n; 
        return SEED ^ 0x464b713e;
    }
}


fn decode(n: usize) -> u32 {
    let m;
    let r;
    let mut x = codgen();
   
    // One's complement of codgen
    x = !x;

    if WORDARR[n] == 0 {
        unsafe {
            BYTEARR[n] = 0;
        }
        return x;
    } else {
        let y = decode(n + 1);
        unsafe {
            m = x.wrapping_sub(y).wrapping_sub(WORDARR[n]);
            BYTEARR[n] = ((m >> 13) & 0xFF) as u8;
        }
        r = (!(codgen())).wrapping_add(1);
        return x.wrapping_add(y).wrapping_add(m).wrapping_add(r).wrapping_add(5);
    
    }
}

//fn _test_codgen() {
//    // seeding with 0x3e944b9f should yield
//    // 891432f9
//    // 4aa1dccc
//    // c54270fa
//    // 9885155f
//    // ce83d1b8 ...
//    let (mut next_seed, mut next_value) = codgen(0x3e944b9f);
//    println!("{:x}", next_value);
//
//    for _ in 0..4 {
//        let (tmp_seed, tmp_value) = codgen(next_seed);
//        next_seed = tmp_seed;
//        next_value = tmp_value;
//        println!("{:x}", next_value);
//        
//    }
//}

static mut BYTEARR: [u8; 4] = [0;4];
static mut SEED: u32 = 0x0e0657c1;
static WORDARR: [u32; 4] = [0x9fdd9158, 0x85715808, 0xac73323a, 0];


fn main() {


    let mut stdout = hio::hstdout().unwrap();
    //let tmp = codgen(0x0e0657c1).0;
    unsafe {
        (*DWT.get()).enable_cycle_counter();
        (*DWT.get()).cyccnt.write(0);
    }
    decode(0);
    let cycle_count;
    unsafe {
       cycle_count = (*DWT.get()).cyccnt.read();
    }
    //println!("{:?}", bytearr as str);
    //unsafe {
    //    for x in BYTEARR.iter() {
    //        write!(stdout, "{}", *x as char).unwrap();
    //    };
    //}
    writeln!(stdout, "{}", cycle_count);
    
}
