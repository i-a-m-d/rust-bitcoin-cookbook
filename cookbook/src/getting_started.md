# Getting Started

Before diving into individual `rust-bitcoin` types, it helps to get a small starter project compiling and running successfully. This chapter shows the smallest practical setup and points out the feature flags you are most likely to want early on.

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

At this point, you should have a project layout that looks like this:

```text
my-rust-bitcoin-project
├── Cargo.toml
└── src
    └── main.rs
```

And the default `src/main.rs` produced by Cargo will look like this:

```rust
fn main() {
    println!("Hello, world!");
}
```

We'll replace that with our first `rust-bitcoin` program later in this chapter.

## Add the crate

Now that you have your project set up, add the `bitcoin` crate with Cargo:

```bash
cargo add bitcoin
```

Alternatively, you can edit `Cargo.toml` directly and add the `bitcoin` dependency set to the version that matches this cookbook:

```toml
[dependencies]
bitcoin = "0.32.8"
```

## Enable useful features when you need them

`rust-bitcoin` has several package features, which let you opt into additional capabilities at compile time.

The first additional features you are most likely to care about are:

- `rand-std`: enables helpers like random private key generation
- `serde`: enables serialization and deserialization support for Bitcoin types

Enable the `rand-std` feature:

```bash
cargo add bitcoin --features rand-std
```

Enable both the `rand-std` and `serde` features:

```bash
cargo add bitcoin --features rand-std,serde
```

## Write a first tiny program

A fun first program is one that generates a brand new testnet keypair and prints the corresponding WIF and address.

Replace the default contents of `src/main.rs` with:

{{#runnable runnable_examples/examples/getting_started/first_tiny_program.rs mode=nondeterministic}}

Then run it:

```bash
cargo run
```

If you use the run button in this book, it will reveal one of several captured sample outputs. Because this example uses randomness, your own locally generated key and address will differ.

This gives you a quick success case built around three important ideas:

- `PrivateKey::generate(Network::Testnet)` creates a new testnet private key using secure randomness
- `to_wif()` prints the key in Wallet Import Format (WIF), which is a common string representation for private keys
- `Address::p2wpkh(...)` derives a native SegWit address from the public key

For a local testnet experiment, it is fine to print the private key like this. In real applications, treat private keys as sensitive material and avoid logging them.
