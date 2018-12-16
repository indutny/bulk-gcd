extern crate bulk_gcd;
extern crate num_cpus;
extern crate rug;

use std::fs;
use std::env;
use rug::Integer;

fn main() {
    let args: Vec<String> = env::args().collect();

    let options = bulk_gcd::ComputeOptions {
        thread_count: num_cpus::get(),
        debug: true,
    };

    match args.len() {
        2 => {
            if options.debug {
                eprintln!("reading file \"{}\"", &args[1]);
            }

            let binary_moduli = fs::read(&args[1])
                .expect(&format!("Module file \"{}\" not found", args[1]));

            let str_moduli = String::from_utf8(binary_moduli).unwrap();

            if options.debug {
                eprintln!("parsing moduli");
            }

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

            if options.debug {
                eprintln!("adding {} padding to moduli", pad_size);
            }

            for _ in 0..pad_size {
                moduli.push(Integer::from(1));
            }

            if options.debug {
                eprintln!("computing gcd");
            }
            let result = bulk_gcd::compute(&moduli, &options);

            result
                .iter()
                .enumerate()
                .for_each(|(i, maybe_gcd)| {
                    match maybe_gcd {
                        None => {
                        },
                        Some(gcd) => {
                            println!("{},{}", i, gcd.to_string_radix(16));
                        }
                    };
                })
        }
        _ => {
            println!("Usage: {} moduli.hex", &args[0]);
        }
    }
}
