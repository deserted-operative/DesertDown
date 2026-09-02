use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use desert_down::parser;
use std::hint::black_box;

static CASES: &[(&str, &str)] = &[
    (
        "Very_small_example",
        include_str!("../example_inputs/Very_small_example.md"),
    ),
    ("Demo", include_str!("../example_inputs/Demo.md")),
    (
        "Demo_1000_lines",
        include_str!("../example_inputs/Demo_1000_lines.md"),
    ),
    (
        "Demo_50000_lines",
        include_str!("../example_inputs/Demo_50000_lines.md"),
    ),
    (
        "Demo_500000_lines",
        include_str!("../example_inputs/Demo_500000_lines.md"),
    ),
    (
        "Demo_1000000_lines",
        include_str!("../example_inputs/Demo_1000000_lines.md"),
    ),
];

fn time_to_ast(c: &mut Criterion) {
    for &(case_name, input) in CASES {
        let mut group = c.benchmark_group(format!("time_to_ast/{case_name}"));

        // get number of bytes in input, to track throughput in a comparable manner to cmark
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_function("DesertDown", |b: &mut criterion::Bencher<'_>| {
            b.iter(|| black_box(parser::parse_input(black_box(input))));
        });

        group.finish();
    }
}

criterion_group!(benches, time_to_ast);
criterion_main!(benches);
