[![GitHub license](https://img.shields.io/github/license/Cythan-Project/Cythan-v3)](https://github.com/Cythan-Project/Cythan-v3/blob/master/LICENSE)
[![Github Workflow](https://img.shields.io/github/workflow/status/Cythan-Project/Cythan-v3/Rust)](https://github.com/Cythan-Project/Cythan-v3/actions)
[![Crate.io](https://img.shields.io/crates/v/cythan)](https://crates.io/crates/cythan)


# Cythan v3
 Cythan is an abstract machine that has been created to be simpler than Turing's one.
 This is the Rust implementation of Cythan.

## Why Rust ?
 - Blazingly fast performances 
 - Low memory foot-print
 - Great ecosystem
 - Concurrency
 - Memory safety
 - WASM compilable

## How to use Cythan in a project

#### Cargo.toml
```
[dependencies]
cythan = "*"
```

#### Example
```rust
use cythan::{BasicCythan,Cythan};
let mut cythan = BasicCythan::new(vec![12,78,15,21,20]);
for _ in 0..20 {
    cythan.next();
}
println!("{}",cythan);
```
## Have found a bug, want to contribute or had an idea ?
Go in the issue section and leave an issue or fix one!
