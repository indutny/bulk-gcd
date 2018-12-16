extern crate rayon;
extern crate rug;

use rayon::prelude::*;
use rug::ops::Pow;
use rug::Integer;

/// Computation options.
pub struct ComputeOptions {
    /// If `true` - debug information will be printed to stderr. Could be used
    /// for monitoring progress
    pub debug: bool,
}

/// Possible computation errors
#[derive(Debug)]
pub enum ComputeError {
    /// Returned when `compute()` (or `compute_with_opts()`) is called with
    /// 0 or 1 moduli. Minimum 2 moduli are required for meaningful operation of
    /// the function.
    NotEnoughModuli,
}

struct ProductTree {
    levels: Vec<Vec<Integer>>,
}

struct RemainderResult {
    remainders: Option<Vec<Integer>>,
    level: Vec<Integer>,
}

fn compute_product_tree(moduli: Vec<Integer>) -> ProductTree {
    // Root
    if moduli.len() == 1 {
        return ProductTree { levels: vec![ moduli ] }
    }

    // Node
    let level = (0..(moduli.len() / 2))
        .into_par_iter()
        .map(|i| {
            Integer::from(&moduli[i * 2] * &moduli[i * 2 + 1])
        })
        .collect();

    let mut res = compute_product_tree(level);
    res.levels.push(moduli);
    return res;
}

fn compute_remainders(tree: ProductTree,
                      options: &ComputeOptions) -> RemainderResult {
    if options.debug {
        eprintln!("computing remainders");
    }

    return tree.levels
        .into_iter()
        .fold(None, |acc, level| {
            if acc.is_none() {
                return Some(RemainderResult {
                    remainders: None,
                    level: level,
                });
            }

            let last = acc.unwrap();

            let previous_results = match last.remainders {
                None => last.level,
                Some(remainders) => remainders,
            };

            let remainders = level.par_iter().enumerate().map(|(i, value)| {
                let parent = &previous_results[i / 2];
                let square = Integer::from(value.pow(2));
                return Integer::from(parent % square);
            }).collect();

            Some(RemainderResult {
                remainders: Some(remainders),
                level: level,
            })
        })
        .unwrap();
}

fn compute_gcds(remainders: &Vec<Integer>,
                moduli: &Vec<Integer>,
                options: &ComputeOptions) -> Vec<Integer> {
    if options.debug {
        eprintln!("computing quotients and gcd");
    }

    // TODO(indutny): parallelize this!
    remainders
        .iter()
        .zip(moduli)
        .map(|(remainder, modulo)| {
            let quotient = Integer::from(remainder / modulo);
            quotient.gcd(modulo)
        })
        .collect()
}

/// Same as `compute()`, but accepts additional compute options argument.
pub fn compute_with_opts(mut moduli: Vec<Integer>, options: &ComputeOptions)
    -> Result<Vec<Option<Integer>>, ComputeError> {
    if moduli.len() < 2 {
        return Err(ComputeError::NotEnoughModuli);
    }

    let original_len = moduli.len();

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
        eprintln!("computing product tree");
    }

    let product_tree = compute_product_tree(moduli);
    let remainder_result = compute_remainders(product_tree, options);
    let mut gcds = compute_gcds(&remainder_result.remainders.unwrap(),
                                &remainder_result.level,
                                options);

    // Remove padding
    gcds.resize(original_len, Integer::from(0));

    let one = Integer::from(1);
    Ok(gcds.into_iter().map(|gcd| {
        if gcd == one {
            None
        } else {
            Some(gcd)
        }
    }).collect())
}

/// Bulk-compute GCD of each modulo in moduli vec with the product of all other
/// moduli in the same list.
///
/// This is an implementation of algorithm by [D. Bernstein][bernstein].
///
/// See: https://factorable.net/weakkeys12.conference.pdf for more details.
///
/// Usage example:
/// ```rust
/// extern crate bulk_gcd;
/// extern crate rug;
///
/// use rug::Integer;
///
/// let moduli = vec![
///     Integer::from(15),
///     Integer::from(35),
///     Integer::from(23),
/// ];
///
/// let result = bulk_gcd::compute(moduli).unwrap();
///
/// assert_eq!(result, vec![
///     Some(Integer::from(5)),
///     Some(Integer::from(5)),
///     None,
/// ]);
/// ```
pub fn compute(moduli: Vec<Integer>)
    -> Result<Vec<Option<Integer>>, ComputeError> {
    compute_with_opts(moduli, &ComputeOptions { debug: false })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_fail_on_zero_moduli() {
        assert!(compute(vec![]).is_err());
    }

    #[test]
    fn it_should_fail_on_single_moduli() {
        assert!(compute(vec![ Integer::new() ]).is_err());
    }

    #[test]
    fn it_should_return_gcd_of_two_moduli() {
        let moduli = vec![ Integer::from(6), Integer::from(15) ];

        let result = compute(moduli).unwrap();
        assert_eq!(result, vec![
            Some(Integer::from(3)),
            Some(Integer::from(3)),
        ]);
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

        let result = compute(moduli).unwrap();

        assert_eq!(result, vec![
            Some(Integer::from(31 * 41)),
            Some(Integer::from(41)),
            None,
            Some(Integer::from(31)),
            Some(Integer::from(131)),
            Some(Integer::from(131)),
        ]);
    }
}
