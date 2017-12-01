# blockchain-rs

Blockchain implementation on Rust

## So, what is this?
This is my first attempt at implementing a very basic blockchain in Rust. It uses simple vectors to store transactions in blocks, serializes to JSON, and uses a very simple algorithm which reminds a little of Bitcoin's hashcash, with SHA256 hashing.
Next features to be added are a functioning REPL for testing the blockchain itself, dumping it to a text file, then recovering it; I also plan to add a way to make transactions over a network, but that might require a lot more study from me.

Please mind that I'm not actually paying attention to the actual currency, only to the blockchain data structure and how it works, while I also take the chance to learn and try to code something in Rust.

This blockchain implementation is largely based on [this article from website HackerNoon](https://hackernoon.com/learn-blockchains-by-building-one-117428612f46), however, I've rewritten it in Rust in the way I could. Plus, since I couldn't rely on Flask, I tried to take the multithreaded approach with a CLI interface of sorts (currently WIP).

## Compile
Since this is a Rust program, you'll find no difficulty building it if you have Cargo installed.

```bash
cd /path/to/folder
cargo build
```

You can also use `cargo run` to execute the program directly.

## License
This program uses the MIT License. Check the file `LICENSE` for details.

## Copyright
(c) 2017 Lucas Vieira
