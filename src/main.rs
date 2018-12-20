extern crate bulk_gcd;
extern crate clap;
extern crate rug;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};
use rug::Integer;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
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
    trace!("opening file \"{}\"", &input);

    let reader = match File::open(&input) {
        Ok(f) => BufReader::new(f),
        Err(err) => {
            eprintln!("Failed to open \"{}\", due to error: \"{}\"", input, err);
            exit(1);
        }
    };

    trace!("reading and parsing moduli");

    let moduli: Vec<Integer> = reader
        .lines()
        .filter_map(|maybe_line| match maybe_line {
            Ok(line) => {
                if line.is_empty() {
                    None
                } else {
                    Some(line)
                }
            }
            Err(err) => {
                eprintln!(
                    "Failed to read line from \"{}\", due to error: \"{}\"",
                    input, err
                );
                exit(1);
            }
        })
        .map(|line| {
            let parse_result = Integer::parse_radix(line, 16).unwrap();
            Integer::from(parse_result)
        })
        .collect();

    trace!("computing gcd on {} moduli", moduli.len());

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
