# blockchain_rust

A blockchain based on proof of work implemented in Rust.

If you want to print the log info in terminal: `RUST_LOG=info cargo run <Subcommand> [Arguments]`

**CLI Command:**

Currently in development stage,we need add `cargo run` in each command.

```zsh
# print chain
cargo run printchain
# reindex unspent-transaction-output set
cargo run reindex
# create wallet
cargo run createwallet
# create blockchain
cargo run createblockchain <address>
# get balance
cargo run getbalance <address>
# list all addresses
cargo run listaddresses
# send amount (if -m is specified, the block will be mined immediately in the same node)
cargo run send <from address> <to address> <amount> -m
# start node server
cargo run startnode <port>
# start mine server
cargo run minernode <port> <address>
```

## Implementation Steps

> [Basic Prototype](https://github.com/Fan03z/blockchain_rust/tree/9b17796ba6efb48f30c1dcc8e8cbc6b5560aeaf3)
>
> [Proof of Work](https://github.com/Fan03z/blockchain_rust/tree/d13850d3c452112de359fd3e931adb08c9d39032)
>
> [Persistence and CLI](https://github.com/Fan03z/blockchain_rust/tree/dee258e333bc6f1c6dea7ba76717e8c4019b696b)
>
> [Transaction_Part1](https://github.com/Fan03z/blockchain_rust/tree/2a294b756e3fad0f7c865cb3c0b70f7e60a7104c)
>
> [Addresses](https://github.com/Fan03z/blockchain_rust/tree/fd1e0ab94ddc0238ef4821d625e6cc3729c75f15)
>
> [Transaction_Part2](https://github.com/Fan03z/blockchain_rust/tree/e71e83ecdfaaea2f5cfaad370b3bdf84827d9a34)
>
> Network (main)

Reference: [blockchain_go](https://github.com/Jeiwan/blockchain_go) [blockchain-rust](https://github.com/yunwei37/blockchain-rust)
