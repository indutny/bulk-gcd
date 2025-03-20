extern crate rayon;
extern crate rug;

use rayon::prelude::*;
use rug::Integer;

pub fn pad_ints(mut list: Vec<Integer>) -> (Vec<Integer>, Vec<usize>) {
    // Pad to the power-of-two len
    let mut desired_size: usize = 1;
    loop {
        if desired_size >= list.len() {
            break;
        }
        desired_size <<= 1;
    }

    while list.len() < desired_size {
        list.push(Integer::from(1))
    }

    let mut enumerated: Vec<(usize, Integer)> = list.into_iter().enumerate().collect();

    // Sort by increasing modulo size so that the smallest moduli (and padding)
    // are on the left, and the large are on the right
    enumerated.par_sort_by(|(_, a), (_, b)| a.cmp(b));

    // Reverse second half of the list so that the biggest moduli are close to
    // the center.
    let half = enumerated.len() / 2;
    enumerated[half..].reverse();

    // At this point the vector should look like this order-wise:
    // [1, ..., N, N, ..., 1 ]
    //
    // When computing the product tree we'll multiply elements from left half
    // with the elements from the right half, balancing the size of resulting
    // products.

    // Keep the original indices so that we could restore the order at the end
    let original_indices: Vec<usize> = enumerated.iter().map(|(index, _value)| *index).collect();
    let sorted: Vec<Integer> = enumerated
        .into_iter()
        .map(|(_index, value)| value)
        .collect();
    (sorted, original_indices)
}

pub fn unpad_ints(
    list: Vec<Integer>,
    original_indices: Vec<usize>,
    original_size: usize,
) -> Vec<Integer> {
    let mut pairs = original_indices
        .into_iter()
        .zip(list)
        .collect::<Vec<(usize, Integer)>>();

    pairs.par_sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

    pairs.truncate(original_size);

    pairs.into_iter().map(|(_, value)| value).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_pad_properly() {
        let input = (0..5).map(|i| Integer::from(10 + i)).rev().collect();

        let expected = vec![
            Integer::from(1),
            Integer::from(1),
            Integer::from(1),
            Integer::from(10),
            Integer::from(14),
            Integer::from(13),
            Integer::from(12),
            Integer::from(11),
        ];
        assert_eq!(pad_ints(input), (expected, vec![5, 6, 7, 4, 0, 1, 2, 3]));
    }

    #[test]
    fn it_should_pad_properly_with_clean_division() {
        let input = (0..6).map(|i| Integer::from(10 + i)).collect();

        let expected = vec![
            Integer::from(1),
            Integer::from(1),
            Integer::from(10),
            Integer::from(11),
            Integer::from(15),
            Integer::from(14),
            Integer::from(13),
            Integer::from(12),
        ];
        assert_eq!(pad_ints(input), (expected, vec![6, 7, 0, 1, 5, 4, 3, 2]));
    }

    #[test]
    fn it_should_skip_padding() {
        let input = (0..4).map(|i| Integer::from(10 + i)).collect();

        let expected = vec![
            Integer::from(10),
            Integer::from(11),
            Integer::from(13),
            Integer::from(12),
        ];
        assert_eq!(pad_ints(input), (expected, vec![0, 1, 3, 2]));
    }

    #[test]
    fn it_should_unpad_properly() {
        let input = vec![
            Integer::from(1),
            Integer::from(10),
            Integer::from(1),
            Integer::from(11),
            Integer::from(1),
            Integer::from(12),
            Integer::from(13),
            Integer::from(14),
        ];
        assert_eq!(
            unpad_ints(input, vec![6, 0, 7, 1, 8, 2, 3, 4], 5),
            vec![
                Integer::from(10),
                Integer::from(11),
                Integer::from(12),
                Integer::from(13),
                Integer::from(14),
            ]
        );
    }
}
