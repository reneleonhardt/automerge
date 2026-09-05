use super::SampledBenchmark;
use benchmark_battery::automerge::transaction::Transactable;
use benchmark_battery::automerge::{AutoCommit, Automerge, ReadDoc, SaveOptions, ROOT};
use benchmark_battery::{
    big_paste_doc, big_random_doc, deep_history_doc, maps_in_maps_doc, poorly_simulated_typing_doc,
    text_splice_100,
};
use std::hint::black_box;

const N: u64 = 100_000;
const LARGE_OPS: i64 = 170_000;

pub fn benchmarks() -> Vec<SampledBenchmark> {
    vec![
        SampledBenchmark::no_setup("load_save", "load_save/load_typing", load_typing),
        SampledBenchmark::no_setup("load_save", "load_save/save_typing", save_typing),
        SampledBenchmark::no_setup("load_save", "load_save/load_big_paste", load_big_paste),
        SampledBenchmark::no_setup("load_save", "load_save/save_big_paste", save_big_paste),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/load_deep_history/1000",
            load_deep_history,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_deep_history/1000",
            save_deep_history,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/load_text_splice_100",
            load_text_splice_100,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_text_splice_100",
            save_text_splice_100,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/load_maps_in_maps",
            load_maps_in_maps,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_maps_in_maps",
            save_maps_in_maps,
        ),
        SampledBenchmark::no_setup("load_save", "load_save/load_big_random", load_big_random),
        SampledBenchmark::no_setup("load_save", "load_save/save_big_random", save_big_random),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/load_170k_ops_1mb",
            load_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/load_incremental_170k_ops_1mb",
            load_incremental_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_170k_ops_1mb",
            save_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_uncached_170k_ops_1mb",
            save_uncached_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_after_170k_ops_1mb",
            save_after_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_incremental_170k_ops_1mb",
            save_incremental_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_after_small_edits_170k_ops_1mb",
            save_after_small_edits_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_incremental_small_edits_170k_ops_1mb",
            save_incremental_small_edits_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_after_to_small_edits_170k_ops_1mb",
            save_after_to_small_edits_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_incremental_to_small_edits_170k_ops_1mb",
            save_incremental_to_small_edits_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_cold_fresh_170k_ops_1mb",
            save_cold_fresh_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/save_nocompress_170k_ops_1mb",
            save_nocompress_170k_ops_1mb,
        ),
        SampledBenchmark::no_setup(
            "load_save",
            "load_save/load_nocompress_170k_ops_1mb",
            load_nocompress_170k_ops_1mb,
        ),
    ]
}

fn load_typing() -> Box<dyn FnMut()> {
    let data = poorly_simulated_typing_doc(N).save();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        black_box(doc);
    })
}

fn save_typing() -> Box<dyn FnMut()> {
    let doc = poorly_simulated_typing_doc(N);
    Box::new(move || {
        let data = doc.save();
        black_box(data);
    })
}

fn load_big_paste() -> Box<dyn FnMut()> {
    let data = big_paste_doc(N).save();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        black_box(doc);
    })
}

fn save_big_paste() -> Box<dyn FnMut()> {
    let doc = big_paste_doc(N);
    Box::new(move || {
        let data = doc.save();
        black_box(data);
    })
}

fn load_text_splice_100() -> Box<dyn FnMut()> {
    let data = text_splice_100(N).save();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        black_box(doc);
    })
}

fn save_text_splice_100() -> Box<dyn FnMut()> {
    let doc = text_splice_100(N);
    Box::new(move || {
        let data = doc.save();
        black_box(data);
    })
}

fn load_maps_in_maps() -> Box<dyn FnMut()> {
    let data = maps_in_maps_doc(N).save();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        black_box(doc);
    })
}

fn save_maps_in_maps() -> Box<dyn FnMut()> {
    let doc = maps_in_maps_doc(N);
    Box::new(move || {
        let data = doc.save();
        black_box(data);
    })
}

fn load_big_random() -> Box<dyn FnMut()> {
    let data = big_random_doc(N).save();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        black_box(doc);
    })
}

fn save_big_random() -> Box<dyn FnMut()> {
    let doc = big_random_doc(N);
    Box::new(move || {
        let data = doc.save();
        black_box(data);
    })
}

fn load_deep_history() -> Box<dyn FnMut()> {
    let data = deep_history_doc(N).save();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        black_box(doc);
    })
}

fn save_deep_history() -> Box<dyn FnMut()> {
    let doc = deep_history_doc(N);
    Box::new(move || {
        let data = doc.save();
        black_box(data);
    })
}

fn load_170k_ops_1mb() -> Box<dyn FnMut()> {
    let data = large_ops_data();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        black_box(doc);
    })
}

fn load_incremental_170k_ops_1mb() -> Box<dyn FnMut()> {
    let data = large_ops_data();
    Box::new(move || {
        let mut doc = Automerge::new();
        doc.load_incremental(&data).unwrap();
        black_box(doc);
    })
}

fn save_170k_ops_1mb() -> Box<dyn FnMut()> {
    let doc = large_ops_doc();
    Box::new(move || {
        let data = doc.save();
        black_box(data);
    })
}

fn save_uncached_170k_ops_1mb() -> Box<dyn FnMut()> {
    let doc = large_ops_doc();
    Box::new(move || {
        let data = doc.save_with_options(SaveOptions {
            deflate: true,
            retain_orphans: false,
        });
        black_box(data);
    })
}

fn save_after_170k_ops_1mb() -> Box<dyn FnMut()> {
    let doc = large_ops_doc();
    Box::new(move || {
        let data = doc.save_after(&[]);
        black_box(data);
    })
}

fn save_incremental_170k_ops_1mb() -> Box<dyn FnMut()> {
    let data = large_ops_data();
    let mut doc = AutoCommit::load(&data).unwrap();
    let mut value = LARGE_OPS;
    Box::new(move || {
        doc.put(ROOT, "value", value).unwrap();
        value += 1;
        let data = doc.save_incremental();
        black_box(data);
    })
}

fn save_after_small_edits_170k_ops_1mb() -> Box<dyn FnMut()> {
    let mut doc = large_ops_doc();
    let mut heads = doc.get_heads();
    let mut value = LARGE_OPS;
    Box::new(move || {
        let mut tx = doc.transaction();
        tx.put(ROOT, "value", format!("{value:010x}")).unwrap();
        tx.commit();
        let data = doc.save_after(&heads);
        heads = doc.get_heads();
        value += 1;
        black_box(data);
    })
}

fn save_incremental_small_edits_170k_ops_1mb() -> Box<dyn FnMut()> {
    let data = large_ops_data();
    let mut doc = AutoCommit::load(&data).unwrap();
    let _ = doc.save_incremental();
    let mut value = LARGE_OPS;
    Box::new(move || {
        doc.put(ROOT, "value", format!("{value:010x}")).unwrap();
        value += 1;
        let data = doc.save_incremental();
        black_box(data);
    })
}

fn save_after_to_small_edits_170k_ops_1mb() -> Box<dyn FnMut()> {
    let mut doc = large_ops_doc();
    let mut heads = doc.get_heads();
    let mut output = Vec::new();
    let mut value = LARGE_OPS;
    Box::new(move || {
        let mut tx = doc.transaction();
        tx.put(ROOT, "value", format!("{value:010x}")).unwrap();
        tx.commit();
        output.clear();
        let written = doc.save_after_to(&heads, &mut output).unwrap();
        heads = doc.get_heads();
        value += 1;
        black_box((written, output.len()));
    })
}

fn save_incremental_to_small_edits_170k_ops_1mb() -> Box<dyn FnMut()> {
    let data = large_ops_data();
    let mut doc = AutoCommit::load(&data).unwrap();
    let _ = doc.save_incremental();
    let mut output = Vec::new();
    let mut value = LARGE_OPS;
    Box::new(move || {
        doc.put(ROOT, "value", format!("{value:010x}")).unwrap();
        value += 1;
        output.clear();
        let written = doc.save_incremental_to(&mut output).unwrap();
        black_box((written, output.len()));
    })
}

fn save_cold_fresh_170k_ops_1mb() -> Box<dyn FnMut()> {
    let data = large_ops_data();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        let saved = doc.save();
        black_box(saved);
    })
}

fn save_nocompress_170k_ops_1mb() -> Box<dyn FnMut()> {
    let doc = large_ops_doc();
    Box::new(move || {
        let data = doc.save_nocompress();
        black_box(data);
    })
}

fn load_nocompress_170k_ops_1mb() -> Box<dyn FnMut()> {
    let data = large_ops_doc().save_nocompress();
    Box::new(move || {
        let doc = Automerge::load(&data).unwrap();
        black_box(doc);
    })
}

fn large_ops_data() -> Vec<u8> {
    let data = large_ops_doc().save();
    assert!(
        (900_000..=1_100_000).contains(&data.len()),
        "fixture size changed: {} bytes",
        data.len()
    );
    data
}

fn large_ops_doc() -> Automerge {
    let mut doc = Automerge::new();
    let mut tx = doc.transaction();
    for value in 0..LARGE_OPS {
        let entropy = (value as u64).wrapping_mul(6_364_136_223_846_793_005);
        tx.put(ROOT, "value", format!("{:010x}", entropy & 0xffffffffff))
            .unwrap();
    }
    tx.commit();
    assert_eq!(doc.stats().num_ops, LARGE_OPS as u64);
    doc
}
