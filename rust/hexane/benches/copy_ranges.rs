use divan::Bencher;
use hexane::{Column, Splice};
use std::time::Duration;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

fn source(len: usize) -> Column<u64> {
    Column::from_values((0..len as u64).map(|value| value / 5).collect())
}

#[divan::bench(max_time = Duration::from_secs(3))]
fn full_copy(bencher: Bencher) {
    bencher
        .with_inputs(|| (Column::from_values(vec![1u64, 2]), source(1_000_000)))
        .bench_local_values(|(mut dst, src)| {
            dst.copy_ranges(
                src,
                [Splice {
                    pos: 1,
                    delete: 0,
                    range: 0..usize::MAX,
                }],
            );
            std::hint::black_box(dst.len());
        });
}

#[divan::bench(max_time = Duration::from_secs(3))]
fn scattered_copy(bencher: Bencher) {
    bencher
        .with_inputs(|| {
            let splices: Vec<Splice> = (0..100)
                .map(|index| Splice {
                    pos: 1,
                    delete: 0,
                    range: (index * 10_000)..(index * 10_000 + 1_000),
                })
                .collect();
            (
                Column::from_values(vec![1u64, 2]),
                source(1_000_000),
                splices,
            )
        })
        .bench_local_values(|(mut dst, src, splices)| {
            dst.copy_ranges(src, splices);
            std::hint::black_box(dst.len());
        });
}
