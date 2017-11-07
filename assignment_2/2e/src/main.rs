extern crate rand;

use std::collections::HashMap;
use std::io;
use std::cmp::Ordering;
use rand::Rng;


fn read_input() -> std::result::Result<u32, String> {

    let mut guess = String::new();

    io::stdin().read_line(&mut guess)
        .expect("Failed to read line");


    match guess.trim().parse() {
         Ok(num) => return Ok(num),
         Err(err) => return Err(String::from(format!("in parsing u32, {:?}", err))),
    };

}

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1, 10);
    let mut attempts = 0;
    let mut history = HashMap::new();


    loop {
        println!("Please input your guess.");

        let guess: u32 = match read_input() {
            Ok(num) => num,
            Err(err) => {   println!("{:?}", err); 
                            continue;
            },
        };
       
        attempts += 1;
        println!("You guessed: {}, nr of attempts: {}", guess, attempts);

        history.insert(attempts, String::from(format!("{:?}", guess))); 

        match guess.cmp(&secret_number) {
            Ordering::Less    => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal   => {
                println!("You win!");
                let history_iter = history.iter();
                for (key, val) in history_iter {
                    println!("{:?}, {:?}", key, val);
                }
                break;
            }
        }
    }
}
