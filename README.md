# blockchain-rs

Blockchain implementation on Rust

## So, what is this?
This is my first attempt at implementing a very basic blockchain in Rust. It uses simple vectors to store transactions in blocks, serializes to JSON, and uses a very simple algorithm which reminds a little of Bitcoin's hashcash, with SHA256 hashing.
It works under a REPL for testing the blockchain itself. The blockchain is locally saved by dumping it to a JSON file, then recovered on the next boot.

Please mind that I'm not actually paying attention to the shared asset -- here portrayed as currency --, only to the blockchain data structure and how it works, while I also take the chance to learn and try to code something in Rust.

This blockchain implementation is largely based on [this article from website HackerNoon](https://hackernoon.com/learn-blockchains-by-building-one-117428612f46), however, I've rewritten it in Rust in the way I could. Plus, since I couldn't rely on Flask, I tried to take the multithreaded approach with a CLI interface of sorts, and manually handling HTTP requests from another thread.

## Compile
Since this is a Rust program, you'll find no difficulty building it if you have Cargo installed.

Please notice that, due to `reqwest`/`rust-ssl` requirements, you'll need OpenSSL's headers installed on order to compile.

```bash
cd /path/to/folder
cargo build
```

You can also use `cargo run` to execute the program directly.

## Usage
The program itself works as a node for the blockchain, and spawns two concurrent processes, in which one of them is used to handle the blockchain, while the other one handles remote HTTP requests.

Run the program from your favorite console; you can use the argument `--port=XXXX` to specify a port different than 3000 to serve as HTTP listening port, replacing `XXXX` with your desired port number.

Once the program runs, you'll be greeted with a prompt, awaiting input. There, you can use the following commands.

### help (WIP)
Shows a general help prompt with the commands.

### mine ALIAS-OR-ID
Mines a new block for the blockchain, then rewards the provided identifier for finding it.

### save FILENAME (WIP)
Saves the blockchain to said filename. If omitted, saves to default `blockchain.json`.

### print
Prints the blockchain entirely to the console in a beautifully indented JSON.

### dump (WIP)
I wonder what this does? Hmmm, we'll discover soon.

### node
Manages nodes on the network. This command has the following subcommands:

#### register ADDRESS
Registers an address (format: `http://127.0.0.1:3000`) as a new node to connect in P2P fashion.
Nodes like this are needed so we can resolve conflicts on our blockchain. The more, the better.

#### new
Creates a new node on the local network.
Since this is basically a node-less identifier, I might remove this command soon and come up with a wallet implementation.

#### alias
Manages node aliases for transactions. I might fork these operations to a wallet command soon.
This is also comprised of the following commands/arguments:

- NEW-ALIAS IDENTIFIER
Creates an alias NEW-ALIAS for the identifier IDENTIFIER. This will come in handy for transfers, so you can avoid typing or pasting the same chain of characters over and over again. Think of this as adding an address to a contacts list.

- show
Shows all registered aliases for this node.

- save FILENAME (WIP)
Saves aliases to FILENAME or, if omitted, to `aliases.json`.
Note that these aliases will be saved once you close the application.

### send (WIP)
Creates a new transaction. Is comprised of two arities:

- AMOUNT DESTINATION (WIP)
Sends AMOUNT to DESTINATION. DESTINATION can be an alias or an identifier.

- AMOUNT SOURCE DESTINATION (WIP)
Sends AMOUNT from SOURCE to DESTINATION. I might remove this option depending on how I'm going to handle the difference between nodes and identifiers in the future.

### resolve
Scans through all registered nodes, using the consensus algorithm, and updates our blockchain.

### quit/exit
You know what it does.
This will also save the blockchain to `blockchain.json`.


## Other relevant links
I did not follow those, but they might be a source for consulting soon, since I want to improve this implementation.
- [Build Your Own Blockchain](http://ecomunsing.com/build-your-own-blockchain)
- [A Blockchain in 200 Lines of Code](https://medium.com/@lhartikk/a-blockchain-in-200-lines-of-code-963cc1cc0e54)


## License
This program uses the MIT License. Check the file `LICENSE` for details.

## Copyright
(c) 2017 Lucas Vieira
