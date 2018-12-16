extern crate rayon;
extern crate rug;

use rayon::prelude::*;
use rug::Integer;

pub struct ComputeOptions {
    pub thread_count: usize,
    pub debug: bool,
}

struct ProductTree {
    levels: Vec<Vec<Integer>>,
}

struct ProductForest {
    head: ProductTree,
    tails: Vec<ProductTree>,
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

fn merge_product_trees(mut trees: Vec<ProductTree>) -> ProductForest {

    let roots = trees.iter_mut().map(|tree| {
        assert!(tree.levels.len() > 0);
        assert_eq!(tree.levels[0].len(), 1);

        tree.levels.remove(0)
    }).flatten().collect();

    return ProductForest {
        head: compute_product_subtree(roots),
        tails: trees,
    }
}

fn compute_product_forest(moduli: &Vec<Integer>,
                          options: &ComputeOptions) -> ProductForest {
    if options.debug {
        eprintln!("compute product forest start");
    }

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

fn compute_partial_remainders(tree: &ProductTree) -> Vec<Integer> {
    return vec![];
}

fn compute_remainders(product_forest: ProductForest,
                      options: &ComputeOptions) -> Vec<Integer> {
    if options.debug {
        eprintln!("compute remainders start");
    }

    let tails: Vec<Integer> = product_forest.tails
        .par_iter()
        .enumerate()
        .map(|(i, tree)| {
            if options.debug {
                eprintln!("thread {}: compute partial remainders start", i);
            }
            let res = compute_partial_remainders(tree);
            if options.debug {
                eprintln!("thread {}: compute partial remainders end", i);
            }
            return res;
        })
        .flatten()
        .collect();

    return vec![];
}

pub fn compute(moduli: &Vec<Integer>,
               options: &ComputeOptions) -> Vec<Option<Integer>> {
    assert!(options.thread_count > 0);
    assert_eq!(moduli.len() % options.thread_count, 0);

    let product_forest = compute_product_forest(moduli, options);
    let remainders = compute_remainders(product_forest, options);

    Vec::new()
}
