use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use rug::integer::Order;
use rug::Integer;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub fn read_from(path: &Path) -> io::Result<Vec<Integer>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut result = Vec::<Integer>::new();

    // 32768 bits
    let mut digits = Vec::<u8>::with_capacity(4096);
    loop {
        let size = match reader.read_u32::<LittleEndian>() {
            Err(err) => {
                if err.kind() == io::ErrorKind::UnexpectedEof {
                    return Ok(result);
                } else {
                    return Err(err);
                }
            }
            Ok(size) => size,
        };

        digits.resize(size as usize, 0);
        reader.read_exact(&mut digits)?;
        result.push(Integer::from_digits(&digits, Order::MsfBe));
    }
}

pub fn write_to(path: &Path, values: &[Integer]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut reader = BufWriter::new(file);

    // 2.56e9 bits
    let mut digits = Vec::<u8>::with_capacity(320_000_000);

    for value in values {
        digits.resize(value.significant_digits::<u8>(), 0);
        value.write_digits(&mut digits, Order::MsfBe);

        reader.write_u32::<LittleEndian>(digits.len() as u32)?;

        reader.write_all(&digits)?;
    }
    Ok(())
}
