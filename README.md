# blockchain_rust

A blockchain based on proof of work implemented in Rust.

If you want to print the log info in terminal: `RUST_LOG=info cargo run <Subcommand> [Arguments]`

**CLI Command:**

Currently in development stage,we need add `cargo run` in each command.

```zsh
# printchain
cargo run printchain
# create blockchain
cargo run createblockchain "[address]"
# get balance
cargo run getbalance "[address]"
# send
cargo run send "[from]" "[to]" "[amount]"
```

Cause the address is unimplemented,address could input string.And a block just has a transaction.Send transaction once,mine block once.

> [Basic Prototype](https://github.com/Fan03z/blockchain_rust/tree/9b17796ba6efb48f30c1dcc8e8cbc6b5560aeaf3)
>
> [Proof of Work](https://github.com/Fan03z/blockchain_rust/tree/d13850d3c452112de359fd3e931adb08c9d39032)
>
> [Persistence and CLI](https://github.com/Fan03z/blockchain_rust/tree/dee258e333bc6f1c6dea7ba76717e8c4019b696b)

Reference: [blockchain_go](https://github.com/Jeiwan/blockchain_go) [blockchain-rust](https://github.com/yunwei37/blockchain-rust)
