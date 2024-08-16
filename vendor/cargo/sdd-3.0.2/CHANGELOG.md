# Changelog

3.0.2

* Make `SDD` much more friendly to `miri`.

3.0.1

* Compatible with the [`miri`](https://github.com/rust-lang/miri) memory leak checker.
* Make `Collectible` private since it is unsafe.
* Remove `Guard::defer` which depends on `Collectible`.
* Remove `prepare`.

2.1.0

* Minor performance optimization.
* Remove `Owned::release`.

2.0.0

* `{Owned, Shared}::release` no longer receives a `Guard`.
* `Link` is now public.

1.7.0

* Add `loom` support.

1.6.0

* Add `Guard::accelerate`.

1.5.0

* Fix `Guard::epoch` to return the correct epoch value.

1.4.0

* `Epoch` is now a 4-state type (3 -> 4).

1.3.0

* Add `Epoch`
* Add `Guard::epoch`.

1.2.0

* Remove `Collectible::drop_and_dealloc`.

1.1.0

* Add `prepare`.

1.0.1

* Relax trait bounds of `Guard::defer_execute`.

1.0.0

* Minor code cleanup.

0.2.0

* Make `Guard` `UnwindSafe`.

0.1.0

* Minor optimization.

0.0.1

* Initial commit: code copied from [`scalable-concurrent-containers`](https://github.com/wvwwvwwv/scalable-concurrent-containers).
