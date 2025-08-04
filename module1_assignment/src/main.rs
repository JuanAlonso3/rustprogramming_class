use std::io;

const FREEZING_WATER_F: f32 = 32.0;

fn fahrenheit_to_celsius(f: f32) -> f32 {
    (f - FREEZING_WATER_F) * 5.0 / 9.0
}

fn celsius_to_fahrenheit(c: f32) -> f32 {
    (c * 9.0 / 5.0) + FREEZING_WATER_F
}

fn is_even(n: i32) -> bool{
    if n%2 == 0{
        true
    }
    else{
        false
    }
}

fn check_guess(guess: i32, secret: i32) -> i32{
    if guess > secret{
        return 1;
    }
    if guess < secret{
        return -1;
    }
    else{
        return 0;
    }
}

fn main() {
    let mut fahrenheit_temp: f32 = 70.0;
    let nums = [1, 2, 3, 4, 5];

   for _i in nums.iter() {
        fahrenheit_temp = fahrenheit_to_celsius(fahrenheit_temp);
        println!("Celsius: {}", fahrenheit_temp);

        fahrenheit_temp = celsius_to_fahrenheit(fahrenheit_temp);
        println!("Fahrenheit: {}", fahrenheit_temp);

        fahrenheit_temp +=1.0;
    }

    println!("================================================================");

    let arr =  [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

     for _j in 0..arr.len() {
        if is_even(arr[_j]) {
            println!("Number {} is even", arr[_j]);
        } else {
            println!("Number {} is odd", arr[_j]);
        }
    }

    println!("================================================================");

    let magic_num = 58;
    println!("Can you guess the magic number 0-100: ");
    
    loop {

        let mut guess = String::new();
        
        let num_guess: i32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input, please enter a number.");
                continue; // restart loop on invalid input
            }
        };

        if check_guess(num_guess, magic_num) == 0{
            println!("Your guess is right the magic number is {}.", num_guess);
        }
        else if check_guess(num_guess, magic_num) == 1{
            println!("Your guess is too high try again.");
        }

        else if check_guess(num_guess, magic_num) == -1{
            println!("Your guess is too low try again.");
        }
        

    }

}
