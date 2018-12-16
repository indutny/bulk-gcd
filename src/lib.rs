extern crate rayon;
extern crate rug;

use rayon::prelude::*;
use rug::Integer;

struct ProductTree {
    levels: Vec<Vec<Integer>>,
}

pub struct ComputeOptions {
    pub thread_count: usize,
    pub debug: bool,
}

fn compute_product_subtree(moduli: Vec<Integer>) -> ProductTree {
    // Root
    if moduli.len() == 1 {
        return ProductTree { levels: vec![ moduli ] }
    }

    // Node
    let level_len = moduli.len() / 2;
    let mut level: Vec<Integer> = Vec::with_capacity(level_len);
    for i in 0..level_len {
        level.push(Integer::from(&moduli[i * 2] * &moduli[i * 2 + 1]));
    }

    let mut res = compute_product_subtree(level);
    res.levels.push(moduli);
    return res;
}

fn merge_product_trees(mut trees: Vec<ProductTree>) -> ProductTree {
    let roots = trees.iter_mut().map(|tree| {
        assert!(tree.levels.len() > 0);
        assert_eq!(tree.levels[0].len(), 1);

        tree.levels.remove(0)
    }).flatten().collect();

    let mut head = compute_product_subtree(roots);

    let child_levels = trees[0].levels.len();
    for _ in 0..child_levels {
        let merged_level = trees.iter_mut().map(|tree| {
            tree.levels.remove(0)
        }).flatten().collect();
        head.levels.push(merged_level);
    }

    return head;
}

fn compute_product_tree(moduli: &Vec<Integer>,
                        options: &ComputeOptions) -> ProductTree {
    let child_trees: Vec<ProductTree> = moduli
        .par_chunks(moduli.len() / options.thread_count)
        .enumerate()
        .map(|(i, chunk)| {
            if options.debug {
                eprintln!("thread {}: compute product tree start", i);
            }
            let res = compute_product_subtree(chunk.to_vec());
            if options.debug {
                eprintln!("thread {}: compute product tree end", i);
            }
            return res;
        })
    .collect();

    if options.debug {
        eprintln!("product tree merge start");
    }
    let res = merge_product_trees(child_trees);
    if options.debug {
        eprintln!("product tree merge end");
    }
    return res;
}

pub fn compute(moduli: &Vec<Integer>,
               options: &ComputeOptions) -> Vec<Option<Integer>> {
    assert!(options.thread_count > 0);
    assert_eq!(moduli.len() % options.thread_count, 0);

    let product_tree = compute_product_tree(moduli, options);

    Vec::new()
}
