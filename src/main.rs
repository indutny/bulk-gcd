extern crate bulk_gcd;
extern crate clap;
extern crate rug;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};
use rug::Integer;
use std::fs;
use std::process::exit;

fn main() {
    env_logger::init();

    let matches = App::with_defaults("bulk-gcd")
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about("Compute bulk GCD of a list of hex RSA moduli")
        .arg(
            Arg::with_name("INPUT")
                .help(concat!(
                    "Input file to use. Must contain hex values of moduli ",
                    "separated by newline ('\\n')"
                ))
                .required(true)
                .index(1),
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    trace!("reading file \"{}\"", &input);

    let str_moduli = match fs::read_to_string(&input) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to read \"{}\", due to error: \"{}\"", input, err);
            exit(1);
        }
    };

    trace!("parsing moduli");

    let moduli: Vec<Integer> = str_moduli
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parse_result = Integer::parse_radix(line, 16).unwrap();
            Integer::from(parse_result)
        })
        .collect();

    trace!("computing gcd");

    let result: Vec<(usize, Integer)> = bulk_gcd::compute(&moduli)
        .unwrap()
        .into_iter()
        .enumerate()
        .filter_map(|(i, opt)| match opt {
            Some(gcd) => Some((i, gcd)),
            None => None,
        })
        .collect();

    if result.is_empty() {
        eprintln!("no results");
        exit(1);
    }

    result.iter().for_each(|(i, gcd)| {
        println!("{},{}", i, gcd.to_string_radix(16));
    });
}
