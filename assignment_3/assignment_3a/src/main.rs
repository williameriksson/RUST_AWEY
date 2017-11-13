


fn codgen(seed :u32) -> (u32, u32) {
    let mut n: u32 = 0;
    let rotate_amount = 30;
    let mut x = seed;
    let y: i32 = seed as i32 >> 6;

    // Count the number of 0's in seed
    for k in 0..32 {
        match seed >> k & 1 {
            0 => n += 1,
            _ => continue, 
        }
    }
    // Rotate seed left by specified amount
    for _k in 0..rotate_amount {
        let msb = (x & 0x80000000) >> 31;
        x <<= 1;
        match msb {
            1 => x = x | 0x00000001,
            _ => continue,
        }
    }

    let new_seed = x ^ (y as u32) ^ n;
    return (new_seed, new_seed ^ 0x464b713e);
}


fn decode(wordarr Vec, bytearr Vec<>) {

}

fn test_codgen() {
    // seeding with 0x3e944b9f should yield
    // 891432f9
    // 4aa1dccc
    // c54270fa
    // 9885155f
    // ce83d1b8 ...
    let (mut next_seed, mut next_value) = codgen(0x3e944b9f);
    println!("{:x}", next_value);

    for _ in 0..4 {
        let (tmp_seed, tmp_value) = codgen(next_seed);
        next_seed = tmp_seed;
        next_value = tmp_value;
        println!("{:x}", next_value);
        
    }
}
fn main() {
    test_codgen();
}
