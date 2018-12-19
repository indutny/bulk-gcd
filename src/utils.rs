extern crate rug;

use rug::Integer;

pub fn pad_ints(list: Vec<Integer>) -> (Vec<Integer>, usize) {
    // Pad to the power-of-two len
    let mut desired_size: usize = 1;
    loop {
        if desired_size >= list.len() {
            break;
        }
        desired_size <<= 1;
    }
    let pad_size = desired_size - list.len();

    if pad_size == 0 {
        return (list, pad_size);
    }

    let pad_every = desired_size / pad_size;
    let mut pad_left = pad_size;

    let mut source_iter = list.into_iter();

    // Insert moduli evenly through the list to make tree more balanced
    let result: Vec<Integer> = (0..desired_size)
        .map(|i| {
            if i % pad_every == 0 && pad_left != 0 {
                pad_left -= 1;
                Integer::from(1)
            } else {
                source_iter.next().unwrap()
            }
        })
        .collect();

    (result, pad_size)
}

pub fn unpad_ints(list: Vec<Integer>, pad_size: usize) -> Vec<Integer> {
    if pad_size == 0 {
        return list;
    }

    let unpad_every = list.len() / pad_size;
    let mut pad_left = pad_size;

    list.into_iter()
        .enumerate()
        .filter_map(|(i, elem)| {
            if i % unpad_every == 0 && pad_left != 0 {
                pad_left -= 1;
                None
            } else {
                Some(elem)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_pad_properly() {
        let input = (0..5).map(|i| Integer::from(10 + i)).collect();

        let expected = vec![
            Integer::from(1),
            Integer::from(10),
            Integer::from(1),
            Integer::from(11),
            Integer::from(1),
            Integer::from(12),
            Integer::from(13),
            Integer::from(14),
        ];
        assert_eq!(pad_ints(input), (expected, 3));
    }

    #[test]
    fn it_should_pad_properly_with_clean_division() {
        let input = (0..6).map(|i| Integer::from(10 + i)).collect();

        let expected = vec![
            Integer::from(1),
            Integer::from(10),
            Integer::from(11),
            Integer::from(12),
            Integer::from(1),
            Integer::from(13),
            Integer::from(14),
            Integer::from(15),
        ];
        assert_eq!(pad_ints(input), (expected, 2));
    }

    #[test]
    fn it_should_skip_padding() {
        let input = (0..4).map(|i| Integer::from(10 + i)).collect();

        let expected = vec![
            Integer::from(10),
            Integer::from(11),
            Integer::from(12),
            Integer::from(13),
        ];
        assert_eq!(pad_ints(input), (expected, 0));
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
            unpad_ints(input, 3),
            vec![
                Integer::from(10),
                Integer::from(11),
                Integer::from(12),
                Integer::from(13),
                Integer::from(14),
            ]
        );
    }

    #[test]
    fn it_should_unpad_properly_with_clean_divison() {
        let input = vec![
            Integer::from(1),
            Integer::from(10),
            Integer::from(11),
            Integer::from(12),
            Integer::from(1),
            Integer::from(13),
            Integer::from(14),
            Integer::from(15),
        ];

        let expected = vec![
            Integer::from(10),
            Integer::from(11),
            Integer::from(12),
            Integer::from(13),
            Integer::from(14),
            Integer::from(15),
        ];
        assert_eq!(unpad_ints(input, 2), expected);
    }

    #[test]
    fn it_should_skip_unpadding() {
        let input = (0..4).map(|i| Integer::from(10 + i)).collect();

        let expected = vec![
            Integer::from(10),
            Integer::from(11),
            Integer::from(12),
            Integer::from(13),
        ];
        assert_eq!(unpad_ints(input, 0), expected);
    }
}
