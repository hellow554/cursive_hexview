Cursive-Hexview
===============

[![BuildStatus](https://github.com/hellow554/cursive_hexview/workflows/Rust/badge.svg)](https://github.com/hellow554/cursive_hexview/actions)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

A simple and basic hexviewer which can be used with [cursive][0].


Documentation
-------------

Can be found on [docs.rs](https://docs.rs/cursive_hexview)

Usage
-----

Just put this into your `Cargo.toml`

```toml
[dependencies]
cursive_hexview = "0.7"
```

and this in your rust code.

```rust
extern crate cursive_hexview;
```


For a minimal example see `examples` folder.

![hexdump_example](doc/hexdump_example.png)


[0]: https://crates.io/crates/cursive
