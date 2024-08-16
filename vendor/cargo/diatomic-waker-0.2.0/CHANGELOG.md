# 0.2.0 (2024-07-28)

- Remove unnecessary `Unpin` bound on `WaitUntil`'s closures.
- Make the crate embedded-friendly with `alloc` as a default, optional feature
  ([#1]).
- Add non-owned counterparts to `WakeSink` and `WakeSource` that can be used
  with `no-alloc` ([#2], [#3]).
- Update and make CI more strict ([#5], [#6]).
- Move `DiatomicWaker` and `WaitUntil` to the root module ([#6]).


[#1]: https://github.com/asynchronics/diatomic-waker/pull/1
[#2]: https://github.com/asynchronics/diatomic-waker/pull/2
[#3]: https://github.com/asynchronics/diatomic-waker/pull/3
[#5]: https://github.com/asynchronics/diatomic-waker/pull/5
[#6]: https://github.com/asynchronics/diatomic-waker/pull/6


# 0.1.0 (2022-10-12)

Initial release
