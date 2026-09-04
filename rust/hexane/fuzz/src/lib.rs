use hexane::{Column, ColumnValueRef, DeltaColumn, DeltaValue, PrefixColumn};
use std::fmt::Debug;

struct Bytes<'a> {
    input: &'a [u8],
    state: u64,
}

impl<'a> Bytes<'a> {
    fn new(input: &'a [u8]) -> Self {
        let mut state = 0x9E37_79B9_7F4A_7C15;
        for &byte in input {
            state ^= u64::from(byte).wrapping_add(0x9E37_79B9_7F4A_7C15);
            state = state.rotate_left(27).wrapping_mul(0x94D0_49BB_1331_11EB);
        }
        Self {
            input,
            state: state | 1,
        }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn usize(&mut self, upper: usize) -> usize {
        if upper == 0 {
            0
        } else {
            (self.next() as usize) % upper
        }
    }

    fn steps(&mut self) -> usize {
        1 + self.input.len().min(128) + self.usize(64)
    }
}

fn run_column<T, F>(input: &[u8], max_segments: usize, mut make: F)
where
    T: ColumnValueRef + Clone + PartialEq + Debug,
    F: FnMut(&mut Bytes<'_>) -> T,
    for<'a> T::Get<'a>: PartialEq + Debug,
{
    let mut bytes = Bytes::new(input);
    let mut column = Column::<T>::with_max_segments(max_segments);
    let mut model = Vec::new();

    for _ in 0..bytes.steps() {
        let position = bytes.usize(model.len() + 1);
        match bytes.usize(4) {
            0 => {
                let value = make(&mut bytes);
                column.insert(position, value.clone());
                model.insert(position, value);
            }
            1 if !model.is_empty() => {
                let position = bytes.usize(model.len());
                column.remove(position);
                model.remove(position);
            }
            2 => {
                let delete = bytes.usize(model.len() - position + 1);
                let values: Vec<T> = (0..bytes.usize(5)).map(|_| make(&mut bytes)).collect();
                column.splice(position, delete, values.iter().cloned());
                model.splice(position..position + delete, values);
            }
            _ => {
                let delete = bytes.usize(model.len() - position + 1);
                let count = bytes.usize(5);
                let value = make(&mut bytes);
                {
                    let mut edit = column.edit_at(position);
                    edit.delete(delete).insert_run(value.clone(), count);
                    edit.finish();
                }
                model.splice(
                    position..position + delete,
                    std::iter::repeat_n(value, count),
                );
            }
        }

        assert_eq!(column.len(), model.len());
        assert_eq!(column.iter().map(T::to_owned).collect::<Vec<_>>(), model);
        column.check_invariants();
        let saved = column.save();
        let loaded = Column::<T>::load(&saved).expect("public column bytes must reload");
        assert_eq!(loaded.iter().map(T::to_owned).collect::<Vec<_>>(), model);
        assert_eq!(loaded.save(), saved);
    }
}

fn run_delta<T, F>(input: &[u8], mut make: F)
where
    T: DeltaValue + Copy + PartialEq + Debug,
    F: FnMut(&mut Bytes<'_>) -> T,
{
    let mut bytes = Bytes::new(input);
    let mut column = DeltaColumn::<T>::new();
    let mut model = Vec::new();

    for _step in 0..bytes.steps() {
        let position = bytes.usize(model.len() + 1);
        let operation = bytes.usize(3);
        match operation {
            0 => {
                let value = make(&mut bytes);
                column.insert(position, value);
                model.insert(position, value);
            }
            1 if !model.is_empty() => {
                let position = bytes.usize(model.len());
                column.remove(position);
                model.remove(position);
            }
            _ => {
                let delete = bytes.usize(model.len() - position + 1);
                let values: Vec<T> = (0..bytes.usize(5)).map(|_| make(&mut bytes)).collect();
                column.splice(position, delete, values.iter().copied());
                model.splice(position..position + delete, values);
            }
        }

        assert_eq!(column.len(), model.len());
        assert_eq!(column.iter().collect::<Vec<_>>(), model);
        column.check_invariants();
        let saved = column.save();
        let loaded = DeltaColumn::<T>::load(&saved).expect("public delta bytes must reload");
        assert_eq!(loaded.iter().collect::<Vec<_>>(), model);
        assert_eq!(loaded.save(), saved);
    }
}

fn run_prefix_u32(input: &[u8]) {
    let mut bytes = Bytes::new(input);
    let mut column = PrefixColumn::<u32>::new();
    let mut model = Vec::new();

    for _ in 0..bytes.steps() {
        let position = bytes.usize(model.len() + 1);
        if bytes.usize(3) == 0 || model.is_empty() {
            let value = (bytes.next() % 32) as u32;
            column.insert(position, value);
            model.insert(position, value);
        } else {
            let position = bytes.usize(model.len());
            column.remove(position);
            model.remove(position);
        }

        let mut prefix = 0u64;
        for (index, &value) in model.iter().enumerate() {
            assert_eq!(column.get_prefix(index), prefix);
            prefix += u64::from(value);
            assert_eq!(column.get_total(index), prefix);
        }
        assert_eq!(column.get_prefix(model.len()), prefix);
        assert_eq!(column.sum_range(0..model.len()), prefix);
        assert_eq!(
            column.iter().map(|item| item.value).collect::<Vec<_>>(),
            model
        );
        let saved = column.save();
        let loaded = PrefixColumn::<u32>::load(&saved).expect("public prefix bytes must reload");
        assert_eq!(
            loaded.iter().map(|item| item.value).collect::<Vec<_>>(),
            model
        );
        assert_eq!(loaded.save(), saved);
    }
}

fn run_prefix_bool(input: &[u8]) {
    let mut bytes = Bytes::new(input);
    let mut column = PrefixColumn::<bool>::new();
    let mut model = Vec::new();

    for _ in 0..bytes.steps() {
        let position = bytes.usize(model.len() + 1);
        if bytes.usize(3) == 0 || model.is_empty() {
            let value = bytes.next().is_multiple_of(2);
            column.insert(position, value);
            model.insert(position, value);
        } else {
            let position = bytes.usize(model.len());
            column.remove(position);
            model.remove(position);
        }
        assert_eq!(
            column.get_prefix(model.len()),
            model.iter().filter(|&&v| v).count()
        );
        assert_eq!(
            column.iter().map(|item| item.value).collect::<Vec<_>>(),
            model
        );
        let saved = column.save();
        let loaded =
            PrefixColumn::<bool>::load(&saved).expect("public bool prefix bytes must reload");
        assert_eq!(
            loaded.iter().map(|item| item.value).collect::<Vec<_>>(),
            model
        );
        assert_eq!(loaded.save(), saved);
    }
}

pub fn run_public_api(input: &[u8]) {
    if input.is_empty() {
        return;
    }
    match input[0] % 6 {
        0 => run_column(input, 2, |bytes| bytes.next() % 4),
        1 => run_column(input, 4, |bytes| bytes.next().is_multiple_of(2)),
        2 => run_column(input, 8, |bytes| {
            if bytes.next().is_multiple_of(3) {
                None
            } else {
                Some(bytes.next() % 4)
            }
        }),
        3 => run_column(input, 8, |bytes| {
            if bytes.next().is_multiple_of(3) {
                None
            } else {
                Some(String::from_utf8(vec![b'a' + (bytes.next() % 8) as u8]).unwrap())
            }
        }),
        4 => run_delta(input, |bytes| (bytes.next() % 256) as i64 - 128),
        _ => {
            run_prefix_u32(input);
            run_prefix_bool(input);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run_public_api;

    #[test]
    fn public_api_accepts_empty_input() {
        run_public_api(&[]);
    }

    #[test]
    fn public_api_smoke_covers_all_modes() {
        for mode in 0..6 {
            let mut input = vec![mode, 1, 3, 5, 8, 13, 21, 34, 55];
            run_public_api(&input);
            input[1] = 0;
            run_public_api(&input);
        }
    }
}
