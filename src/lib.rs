extern crate rayon;
extern crate rug;

use rayon::prelude::*;
use rug::ops::Pow;
use rug::Integer;

pub struct ComputeOptions {
    pub debug: bool,
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

pub fn compute(mut moduli: Vec<Integer>, options: &ComputeOptions)
    -> Result<Vec<Option<Integer>>, &str> {
    if moduli.len() < 2 {
        return Err("At least two moduli are required to run the algorithm");
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_OPTIONS: ComputeOptions = ComputeOptions { debug: false };

    #[test]
    fn it_should_fail_on_zero_moduli() {
        assert!(compute(vec![], &TEST_OPTIONS).is_err());
    }

    #[test]
    fn it_should_fail_on_single_moduli() {
        assert!(compute(vec![ Integer::new() ], &TEST_OPTIONS).is_err());
    }

    #[test]
    fn it_should_return_gcd_of_two_moduli() {
        let moduli = vec![ Integer::from(6), Integer::from(15) ];

        let result = compute(moduli, &TEST_OPTIONS).unwrap();
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

        let result = compute(moduli, &TEST_OPTIONS).unwrap();

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
