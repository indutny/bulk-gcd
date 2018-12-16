extern crate bulk_gcd;
extern crate rug;

#[macro_use]
extern crate log;
extern crate env_logger;

use std::fs;
use std::env;
use rug::Integer;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} moduli.hex", &args[0]);
        std::process::exit(1);
    }

    trace!("reading file \"{}\"", &args[1]);

    let binary_moduli = fs::read(&args[1])
        .expect(&format!("Module file \"{}\" not found", args[1]));

    let str_moduli = String::from_utf8(binary_moduli).unwrap();

    trace!("parsing moduli");

    let moduli: Vec<Integer> = str_moduli.split('\n')
        .filter(|line| line.len() != 0)
        .map(|line| {
            let parse_result = Integer::parse_radix(line, 16).unwrap();
            Integer::from(parse_result)
        }).collect();

    trace!("computing gcd");

    let result: Vec<(usize, Integer)> =
        bulk_gcd::compute(moduli)
        .unwrap()
        .into_iter()
        .enumerate()
        .filter_map(|(i, opt)| {
            match opt {
                Some(gcd) => Some((i, gcd)),
                None => None
            }
        })
        .collect();

    if result.len() == 0 {
        eprintln!("no results");
        std::process::exit(1);
    }

    result
        .iter()
        .for_each(|(i, gcd)| {
            println!("{},{}", i, gcd.to_string_radix(16));
        });
}
