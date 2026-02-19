# Generating keys
## 1.1 Bitcoin keys
For this recipe, the `rand-std` feature is required.

We'll make sure the `bitcoin` crate is built with `rand-std` enabled:

``` bash
cargo add bitcoin --features rand-std
```

``` rust
use bitcoin::{PrivateKey, NetworkKind};
use bitcoin::secp256k1::{Secp256k1};

fn main() {
    let secp = Secp256k1::new();
    let network = NetworkKind::Test;
    let ecdsa_sk = PrivateKey::generate(network);
    let ecdsa_pk = ecdsa_sk.public_key(&secp);
}
```