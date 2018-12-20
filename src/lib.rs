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
//! let result = bulk_gcd::compute(&moduli).unwrap();
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
extern crate rayon;
extern crate rug;

#[macro_use]
extern crate log;
extern crate env_logger;

mod utils;

use rayon::prelude::*;
use rug::Integer;
use utils::*;

/// Possible computation errors
#[derive(Debug, PartialEq)]
pub enum ComputeError {
    /// Returned when `compute()` is called with 0 or 1 moduli
    /// Minimum 2 moduli are required for meaningful operation of the function.
    NotEnoughModuli,
}

struct ProductTree {
    levels: Vec<Vec<Integer>>,
}

fn compute_product_tree(moduli: Vec<Integer>) -> ProductTree {
    // Root
    if moduli.len() == 1 {
        return ProductTree {
            levels: vec![moduli],
        };
    }

    // Node
    let level = (0..(moduli.len() / 2))
        .into_par_iter()
        .map(|i| Integer::from(&moduli[i * 2] * &moduli[i * 2 + 1]))
        .collect();

    let mut res = compute_product_tree(level);
    res.levels.push(moduli);
    res
}

fn compute_remainders(tree: ProductTree) -> Option<Vec<Integer>> {
    let level_count = tree.levels.len() - 1;
    trace!("computing remainders for {} levels", level_count);

    tree.levels
        .into_iter()
        .enumerate()
        .fold(None, |maybe_parent, (level, current)| {
            let parent = match maybe_parent {
                None => {
                    return Some(current);
                },
                Some(parent) => parent,
            };

            trace!("computing remainder level {}/{}", level, level_count);
            let remainders = current
                .into_par_iter()
                .enumerate()
                .map(|(i, mut value)| {
                    // value = parent[i / 2] % (value ** 2)
                    value.square_mut();

                    &parent[i / 2] % value
                })
                .collect();

            Some(remainders)
        })
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
/// let result = bulk_gcd::compute(&moduli).unwrap();
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
///     bulk_gcd::compute(&[]).unwrap_err(),
///     bulk_gcd::ComputeError::NotEnoughModuli
/// );
/// ```
///
pub fn compute(moduli: &[Integer]) -> Result<Vec<Option<Integer>>, ComputeError> {
    if moduli.len() < 2 {
        return Err(ComputeError::NotEnoughModuli);
    }

    // Pad to the power-of-two len
    let (padded_moduli, pad_size) = pad_ints(moduli.to_vec());
    trace!("added {} padding to moduli", pad_size);

    trace!("computing product tree");
    let tree = compute_product_tree(padded_moduli);
    let remainders = compute_remainders(tree);

    let gcds = compute_gcds(&unpad_ints(remainders.unwrap(), pad_size), moduli);

    Ok(gcds
        .into_iter()
        .map(|gcd| if gcd == 1 { None } else { Some(gcd) })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_fail_on_zero_moduli() {
        assert!(compute(&[]).is_err());
    }

    #[test]
    fn it_should_fail_on_single_moduli() {
        assert!(compute(&[Integer::new()]).is_err());
    }

    #[test]
    fn it_should_return_gcd_of_two_moduli() {
        let moduli = [Integer::from(6), Integer::from(15)];

        let result = compute(&moduli).unwrap();
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

        let result = compute(&moduli).unwrap();

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
