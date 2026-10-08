# Rust catalogue measurement, 2026-10-08

The Release example `rust/storage/examples/benchmark_catalog.rs` runs five fresh
databases for each corpus size, with 200 query samples per pass: 1,000 samples per
size. The synthetic corpus follows the incumbent `tool/benchmark_library.dart`
IDs, titles, authors, favourite distribution and ordering. Each pass repeats its
20-query pattern ten times. A separate timed series includes both repository
query and count, as the current application command does.

| Documents | Worst query+count p95 among five passes | Budget |
|---|---:|---:|
| 5,000 | 1.995 ms | 50 ms |
| 25,000 | 5.145 ms | 50 ms |
| 100,000 | 19.562 ms | 50 ms |

The paged query materializes at most 80 documents. The measured series satisfies
the specified catalogue budget on this host. The complete raw report, including
p50 and each pass, is `rust-catalog-benchmark-2026-10-08.json`.

Reproduce with:

```powershell
cargo run --release -p yamantaka-storage --example benchmark_catalog --offline
```

## Subsequent integrated query measurement

The repeat after integrating the extended query contract, generated at
2026-10-08T18:19:32Z, is preserved separately in
`rust-catalog-benchmark-stage2-2026-10-08.json`. The same five-pass pattern yielded
worst query+count p95 of **2.451, 6.546 and 24.782 ms** for 5k, 25k and 100k.
All satisfy 50 ms for this measured pattern. The original report above remains
historical evidence, not the latest measurement. Facet and every-sort stress are
not included. This does not establish a statistically attributable regression
or improvement across different host activity.

This measures the repository only. It does not measure UI frames, GPU composition,
PDF rasterization, memory or process startup. It does not establish a Flutter/Rust
improvement: Flutter analysis and 330 tests now pass on this host, but the
complete paired application sequences remain pending. The historical Flutter benchmark also includes facet work in its
first-load measurement, which the current Rust section does not yet implement;
those first-load numbers must not be treated as equivalent.
