use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use desert_down::{parser, parser::LinkPermissions};
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

fn time_to_html(c: &mut Criterion) {
    for &(case_name, input) in CASES {
        let mut group = c.benchmark_group(format!("time_to_html/{case_name}"));

        // Get the number of bytes in the input, to track throughput comparably
        group.throughput(Throughput::Bytes(input.len() as u64));

        group.bench_function("DesertDown", |b| {
            b.iter(|| {
                let ast = parser::parse_input(black_box(input));

                let mut html = Vec::new();

                ast.ast_to_html_to_array(black_box(input), &mut html, LinkPermissions::Allowed);

                black_box(html)
            });
        });

        group.finish();
    }
}

criterion_group!(benches, time_to_html);
criterion_main!(benches);
