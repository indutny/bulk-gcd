extern crate bulk_gcd;
extern crate clap;
extern crate rug;

#[macro_use]
extern crate log;
extern crate env_logger;

use clap::{App, Arg};
use rug::Integer;
use std::path::{Path, PathBuf};
use std::process::exit;

fn main() -> std::io::Result<()> {
    env_logger::init();

    let matches = App::new("bulk-gcd")
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

    let moduli = bulk_gcd::fs::read_from(Path::new(input))?;

    let cache_dir: Option<PathBuf> = std::env::var_os("CACHE_DIR").map(PathBuf::from);

    if let Some(ref dir) = cache_dir {
        std::fs::create_dir_all(dir)?;
    }

    let result: Vec<(usize, Integer)> = bulk_gcd::compute(&moduli, cache_dir.as_deref())
        .unwrap()
        .into_iter()
        .enumerate()
        .filter_map(|(i, opt)| opt.map(|gcd| (i, gcd)))
        .collect();

    if result.is_empty() {
        eprintln!("no results");
        exit(0);
    }

    for (i, gcd) in result {
        println!("i={} divisor={:x} moduli={:x}", i, gcd, moduli[i]);
    }
    Ok(())
}
