# HOPR PR #8700 — raw Criterion records

Paired measurements for base `ff0076efaa` vs head `be3078a607`.

- `raw_samples.tar.gz` — 96 `sample.json` files, one per harness/group/pass, holding the
  individual per-sample times (`times`) and iteration counts (`iters`) Criterion recorded.
  Layout: `rawsamples/<harness>_<side>_<pass>/<group>.json`.
- `estimates.json` — the `mean.point_estimate` (µs) from each run's `estimates.json`.
- `sample_counts.json` — number of samples actually collected per group. The `packet_bench`
  groups set `sample_size(100_000)` in the harness, which overrides `--sample-size`; the
  `codec_bench` groups took the CLI value of 20.

Commands, machine and pinning are in the PR threads at
https://github.com/hoprnet/hoprnet/pull/8700
