# Rust plugin

[Rust](https://www.rust-lang.org/) WASM plugin for [proto](https://github.com/moonrepo/proto).

```shell
proto install rust
```

## Caveats

If you're familiar with Rust, you most likely use [rustup](https://rustup.rs), a Rust specific toolchain manager. This overlaps heavily with how proto works, so instead of proto reinventing the wheel here, we simply call `rustup` under the hood. Because of this, be aware of the following when using Rust in proto:

- Rust and Cargo must be pre-installed on your system.
- The `~/.cargo/bin` directory must be in your `PATH`.
- We don't install Rust to `~/.proto/tools/rust` but instead reference `~/.rustup/toolchains`.
- We don't create shims for `cargo`, `rustup`, etc.

Since we don't create shims for `cargo`, `rustup`, etc, we can't detect Rust versions at runtime. However, `rustup` supports this through the
[`rust-toolchain.toml`](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file) file. We suggest using this file.

## Contributing

Build the plugin:

```shell
cargo build --target wasm32-wasi
```

Test the plugin by running `proto` commands. Requires proto >= v0.14.

```shell
proto install rust-test
proto list-remote rust-test
```
