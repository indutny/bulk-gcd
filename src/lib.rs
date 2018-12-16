extern crate rug;
extern crate crossbeam_utils;

use crossbeam_utils::thread;
use rug::Integer;

struct ProductTree {
    levels: Vec<Vec<Integer>>,
}

fn compute_product_subtree(moduli: &Vec<Integer>) -> ProductTree {
    // Root
    if moduli.len() == 1 {
        return ProductTree { levels: vec![ moduli.clone() ] }
    }

    // Node
    let level_len = moduli.len() / 2;
    let mut level: Vec<Integer> = Vec::with_capacity(level_len);
    for i in 0..level_len {
        level.push(Integer::from(&moduli[i] * &moduli[i + 1]));
    }

    let mut res = compute_product_subtree(&level);
    res.levels.insert(0, level);
    res
}

fn compute_product_tree(moduli: &Vec<Integer>,
                        thread_count: usize) -> ProductTree {
    let child_trees: Vec<ProductTree> = thread::scope(|scope| {
        moduli
            .chunks(moduli.len() / thread_count)
            .map(|chunk| {
                scope.spawn(move |_| {
                    compute_product_subtree(&chunk.to_vec())
                })
            })
            .map(|handle| handle.join().unwrap())
            .collect()
    }).unwrap();

    ProductTree { levels: Vec::new() }
}

pub fn compute(moduli: &Vec<Integer>,
               thread_count: usize) -> Vec<Option<Integer>> {
    assert_eq!(moduli.len() % thread_count, 0);

    let product_tree = compute_product_tree(moduli, thread_count);

    Vec::new()
}
