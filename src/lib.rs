//! # bulk-gcd
//!
//! This crate computes GCD of each integer in moduli list with the product of
//! all integers in the same list using [fast algorithm][bernstein] by
//! D. Bernstein.
//!
//! See: [this paper][that paper] for more details and motivation.
//!
//! Usage example:
//! ```rust
//! extern crate bulk_gcd;
//! extern crate rug;
//!
//! use rug::Integer;
//!
//! let moduli = [
//!     Integer::from(15),
//!     Integer::from(35),
//!     Integer::from(23),
//! ];
//!
//! let result = bulk_gcd::compute(&moduli, None).unwrap();
//!
//! assert_eq!(
//!     result,
//!     vec![
//!         Some(Integer::from(5)),
//!         Some(Integer::from(5)),
//!         None,
//!     ]
//! );
//! ```
//!
//! [bernstein]: https://cr.yp.to/factorization/smoothparts-20040510.pdf
//! [that paper]: https://factorable.net/weakkeys12.conference.pdf
extern crate byteorder;
extern crate rayon;
extern crate rug;

#[macro_use]
extern crate log;
extern crate env_logger;

pub mod fs;
mod utils;

use rayon::prelude::*;
use rug::integer::IntegerExt64;
use rug::ops::MulFrom;
use rug::Integer;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::path::Path;
use utils::*;

/// Possible computation errors
#[derive(Debug, PartialEq)]
pub enum ComputeError {
    /// Returned when `compute()` is called with 0 or 1 moduli
    /// Minimum 2 moduli are required for meaningful operation of the function.
    NotEnoughModuli,
}

impl Display for ComputeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ComputeError")
    }
}

impl Error for ComputeError {
    fn description(&self) -> &str {
        match self {
            ComputeError::NotEnoughModuli => "Not enough moduli",
        }
    }
}

pub type ComputeResult = Result<Vec<Option<Integer>>, ComputeError>;

fn get_bounds(list: &[Integer]) -> (u64, u64) {
    let min = list
        .iter()
        .fold(u64::MAX, |acc, value| u64::min(acc, value.signed_bits_64()));
    let max = list
        .iter()
        .fold(u64::MIN, |acc, value| u64::max(acc, value.signed_bits_64()));
    (min, max)
}

fn compute_products(
    mut moduli: Vec<Integer>,
    target_len: usize,
    cache_dir: Option<&Path>,
) -> Vec<Integer> {
    if let Some(cache_dir) = cache_dir {
        let path = cache_dir.join(format!("{}.bin", target_len));
        trace!("trying reading products from {}", path.display());

        if let Ok(result) = fs::read_from(path.as_path()) {
            return result;
        }
    }

    while moduli.len() > target_len {
        trace!("computing products {} -> {}", moduli.len(), target_len);

        // See pad_ints() in utils.rs
        // The list is sorted and the second half is reversed so that the
        // smallest moduli is at [0] and the largest is at [half]
        let half = moduli.len() / 2;
        let (left, right) = moduli.split_at_mut(half);

        left.par_iter_mut()
            .zip(right.par_iter_mut())
            .for_each(|(small, big)| small.mul_from(std::mem::take(big)));
        moduli.truncate(half);

        if let Some(cache_dir) = cache_dir {
            if moduli.len() <= target_len {
                continue;
            }

            let (min, max) = get_bounds(&moduli);

            let path = cache_dir.join(format!("{}.bin", moduli.len()));
            trace!(
                "writing products (min={}, max={}) to {}",
                min,
                max,
                path.display()
            );
            fs::write_to(path.as_path(), &moduli).unwrap();
        }
    }
    moduli
}

fn compute_remainders(moduli: Vec<Integer>, cache_dir: Option<&Path>) -> Vec<Integer> {
    let mut pre_last = Some(compute_products(moduli.clone(), 2, cache_dir));
    let mut remainders = compute_products(pre_last.clone().unwrap(), 1, cache_dir);

    let mut depth = 2;
    while depth <= moduli.len() {
        let mut current = pre_last
            .take()
            .unwrap_or_else(|| compute_products(moduli.clone(), depth, cache_dir));

        let (min, max) = get_bounds(&current);
        trace!(
            "computing remainder level {} (min={}, max={})",
            depth,
            min,
            max
        );

        let compute = |(i, value): (usize, &mut Integer)| {
            // value = remainders[i % remainders.len()] % (value ** 2)
            let parent = &remainders[i % remainders.len()];

            // Don't compute square if the divisor is bigger than the divisor
            if value.signed_bits_64() * 2 > parent.signed_bits_64() {
                value.clone_from(parent);
            } else {
                value.square_mut();
                value.modulo_from(parent);
            }
        };

        // First levels use most memory so don't par_iter yet
        if current.len() <= 32 {
            current.iter_mut().enumerate().for_each(compute);
        } else {
            current.par_iter_mut().enumerate().for_each(compute);
        }

        remainders = current;
        depth *= 2;
    }

    remainders
}

fn compute_gcds(remainders: &[Integer], moduli: &[Integer]) -> Vec<Integer> {
    trace!("computing quotients and gcd");
    remainders
        .par_iter()
        .zip(moduli.par_iter())
        .map(|(remainder, modulo)| {
            let quotient = Integer::from(remainder / modulo);
            quotient.gcd(modulo)
        })
        .collect()
}

/// Compute GCD of each integer in the `moduli` with all other integers in it.
///
/// Usage example:
/// ```rust
/// extern crate bulk_gcd;
/// extern crate rug;
///
/// use rug::Integer;
///
/// let moduli = [
///     Integer::from(15),
///     Integer::from(35),
///     Integer::from(23),
///     Integer::from(49),
/// ];
///
/// let result = bulk_gcd::compute(&moduli, None).unwrap();
///
/// assert_eq!(
///     result,
///     vec![
///         Some(Integer::from(5)),
///         Some(Integer::from(35)),
///         None,
///         Some(Integer::from(7)),
///     ]
/// );
/// ```
///
/// NOTE: Minimum 2 `moduli` are required for running the algorithm, otherwise
/// `NotEnoughModuli` error is returned:
///
/// ```rust
/// extern crate bulk_gcd;
/// extern crate rug;
///
/// use rug::Integer;
///
/// assert_eq!(
///     bulk_gcd::compute(&[], None).unwrap_err(),
///     bulk_gcd::ComputeError::NotEnoughModuli
/// );
/// ```
///
pub fn compute(moduli: &[Integer], cache_dir: Option<&Path>) -> ComputeResult {
    if moduli.len() < 2 {
        return Err(ComputeError::NotEnoughModuli);
    }

    trace!("starting with {} moduli", moduli.len());

    // Pad to the power-of-two len
    let (padded_moduli, original_indices) = pad_ints(moduli.to_vec());
    trace!("padded to {}", padded_moduli.len());

    let remainders = compute_remainders(padded_moduli.clone(), cache_dir);

    let gcds = compute_gcds(&remainders, &padded_moduli);

    let unpadded_gcds = unpad_ints(gcds, original_indices, moduli.len());

    Ok(unpadded_gcds
        .into_iter()
        .map(|gcd| if gcd == 1 { None } else { Some(gcd) })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_fail_on_zero_moduli() {
        assert!(compute(&[], None).is_err());
    }

    #[test]
    fn it_should_fail_on_single_moduli() {
        assert!(compute(&[Integer::new()], None).is_err());
    }

    #[test]
    fn it_should_return_gcd_of_two_moduli() {
        let moduli = [Integer::from(6), Integer::from(15)];

        let result = compute(&moduli, None).unwrap();
        assert_eq!(
            result,
            vec![Some(Integer::from(3)), Some(Integer::from(3)),]
        );
    }

    #[test]
    fn it_should_find_gcd_for_many_moduli() {
        let moduli = vec![
            Integer::from(31 * 41),
            Integer::from(41),
            Integer::from(61),
            Integer::from(71 * 31),
            Integer::from(101 * 131),
            Integer::from(131 * 151),
        ];

        let result = compute(&moduli, None).unwrap();

        assert_eq!(
            result,
            vec![
                Some(Integer::from(31 * 41)),
                Some(Integer::from(41)),
                None,
                Some(Integer::from(31)),
                Some(Integer::from(131)),
                Some(Integer::from(131)),
            ]
        );
    }
}
