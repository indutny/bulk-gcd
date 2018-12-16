extern crate bulk_gcd;
extern crate num_cpus;
extern crate rug;

use std::fs;
use std::env;
use rug::Integer;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        2 => {
            let binary_moduli = fs::read(&args[1])
                .expect(&format!("Module file \"{}\" not found", args[1]));

            let str_moduli = String::from_utf8(binary_moduli).unwrap();

            let mut moduli: Vec<Integer> = str_moduli.split('\n')
                .filter(|line| line.len() != 0)
                .map(|line| {
                    let parse_result = Integer::parse_radix(line, 16).unwrap();
                    Integer::from(parse_result)
                }).collect();

            // Pad to the power-of-two len
            let mut pad_size: usize = 1;
            loop {
                if pad_size >= moduli.len() {
                    break;
                }
                pad_size <<= 1;
            }
            pad_size -= moduli.len();

            for _ in 0..pad_size {
                moduli.push(Integer::from(1));
            }

            let result = bulk_gcd::compute(&moduli, num_cpus::get());
            println!("{}", result.len());
        }
        _ => {
            println!("Usage: {} moduli.hex", &args[0]);
        }
    }
}
