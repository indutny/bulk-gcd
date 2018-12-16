# bulk-gcd
[![Build Status](https://secure.travis-ci.org/indutny/bulk-gcd.svg)](http://travis-ci.org/indutny/bulk-gcd)

This package provides and implementation of bulk GCD (Greatest Common Divisor)
algorithm by [D. Bernstein][bernstein].

## Why?

GCD is useful for identifying weak keys in a large set of RSA keys. Such
sets were collected by researches (e.g. [this paper][that paper]). In order to
find weak keys a pairwise GCD has to be executed on all RSA moduli (i.e.
products of two primes `P` and `Q`, pertaining to each RSA private key).
However, each separate GCD computation takes considerable amount of time and the
naive pairwise process doesn't scale well (`O(n^2)`).

Instead of doing this search in a brute-force way, this module employs clever
algorithm by [D. Bernstein][bernstein], that finds GCD of each moduli with a
product of all other moduli. Through introduction of product and remainder
trees, the computational cost becomes logarithmic instead of quadratic, which
results in dramatic drop of the execution time.

## Quick example

```rust
extern crate bulk_gcd;
extern crate rug;

use rug::Integer;

let moduli = vec![
    Integer::from(15),
    Integer::from(35),
    Integer::from(23),
];

let result = bulk_gcd::compute(moduli).unwrap();

assert_eq!(result, vec![
    Some(Integer::from(5)),
    Some(Integer::from(5)),
    None,
]);
```

## Using bulk-gcd

`bulk-gcd` is available on [crates.io][crates]. To use `bulk-gcd` in your crate,
add it as a dependency inside [Cargo.toml][cargo doc]:

```
[dependencies]
bulk-gcd = "^1.0.0"
```

You also need to declare it by adding this to your crate root (usually
`lib.rs` or `main.rs`):

```rust
extern crate bulk_gcd;
```

## Credits

Huge thanks to [Michael Grunder][1] for helping me make threads work in Rust.

[bernstein]: https://cr.yp.to/factorization/smoothparts-20040510.pdf
[that paper]: https://factorable.net/weakkeys12.conference.pdf
[crates]: https://crates.io/crates/bulk-gcd
[cargo doc]: https://doc.rust-lang.org/cargo/guide/dependencies.html
[1]: https://github.com/michael-grunder
