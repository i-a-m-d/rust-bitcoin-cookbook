# Getting Started

Before diving into individual `rust-bitcoin` types, it helps to get one tiny project compiling and running successfully. This chapter shows the smallest practical setup and points out the feature flags you are most likely to want early on.

## Create a project

If you are starting from nothing, first create a directory for your project, change into it, and initialize a new Cargo package:

```bash
mkdir my-rust-bitcoin-project
cd my-rust-bitcoin-project
cargo init
```

Or use the more common `cargo new` to start:

```bash
cargo new my-rust-bitcoin-project
cd my-rust-bitcoin-project
```

## Add the crate

Now that you have your project set up, add the `bitcoin` crate with Cargo:

```bash
cargo add bitcoin
```

Alternatively, you can edit `Cargo.toml` directly and add the `bitcoin` dependency with the version constraint that matches this cookbook:

```toml
[dependencies]
bitcoin = "0.32.8"
```

## Enable useful features when you need them