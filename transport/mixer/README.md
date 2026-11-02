# hopr-transport-mixer

Mixer functionality responsible for mixing and delaying packets with a channel-like interface

Part of the [HOPR](https://hoprnet.org/) protocol implementation.

## Specification

- [RFC-0006: HOPR Mixer](https://github.com/hoprnet/rfc/blob/main/rfcs/RFC-0006-hopr-mixer/0006-hopr-mixer.md)

## Configuration upgrade

Mixer tuning now belongs under `protocol.mixer.mixer_type`. The former top-level
`min_delay`, `delay_range`, and `metric_delay_window` keys are rejected during deserialization.
The metric delay window is derived from the selected engine's delay bound.

The transport's default engine is now Poisson, with a 20 ms hard bound and an approximately
4.14 ms mean delay in bounded-latency mode. To retain the previous uniform 0–20 ms behavior,
select Uniform explicitly (requires the `uniform-channel` feature):

```yaml
protocol:
  mixer:
    capacity: 20000
    mixer_type:
      Uniform:
        min_delay: 0ms
        delay_range: 20ms
```

For the Poisson engine (requires the `poisson` feature):

```yaml
protocol:
  mixer:
    capacity: 20000
    mixer_type:
      Poisson:
        max_delay: 20ms
        miss_probability: 0.01
        target_occupancy: 0
```

`miss_probability` controls truncation of the exponential release tags; it is not a target
rate of packets exceeding `max_delay`. A nonzero `target_occupancy` enables constant-privacy
mode; `0` keeps the mean delay independent of ingress load.

## License

GPL-3.0-only
