# blockchain-rs

Blockchain implementation on Rust

## So, what is this?
This is my first attempt at implementing a very basic blockchain in Rust. It uses simple vectors to store transactions in blocks, serializes to JSON, and uses a very simple algorithm which reminds a little of Bitcoin's hashcash, with SHA256 hashing.
It works under a REPL for testing the blockchain itself. The blockchain is locally saved by dumping it to a JSON file, then recovered on the next boot.

Please mind that I'm not actually paying attention to the shared asset -- here portrayed as currency --, only to the blockchain data structure and how it works, while I also take the chance to learn and try to code something in Rust.

This blockchain implementation is largely based on [this article from website HackerNoon](https://hackernoon.com/learn-blockchains-by-building-one-117428612f46), however, I've rewritten it in Rust in the way I could. Plus, since I couldn't rely on Flask, I tried to take the multithreaded approach with a CLI interface of sorts, and manually handling HTTP requests from another thread.

## What does it do?
I have set some goals for this project. Although this is probably not the best implementation of a blockchain, I aim to

- Provide a basic implementation for a blockchain, not worrying about security since it is being done for educational purposes;
- Although there is almost no focus on security, I may implement some security measures here and there to learn about them;
- The blockchain will be able to sync with remote nodes;
- Simple wallets can be generated for testing purposes, to test mining and transactions;
- Some algorithms may follow some standards, but not entirely. For example, as per today (12/8/2017), the wallets are a set of five addresses, generated much like v1 of Bitcoin, or at least how it is described on the wiki (see links below). However, the seed for address generation is an UUID instead of a public key. Since generating the pair requires a few security considerations for doing it right, I just didn't do it for now, since I really don't care about it right now;
- Interact with the blockchain from a very basic HTTP protocol, mostly used for consensus only;
- Interact with the blockchain using a console interface, a REPL;
- Try to follow the Rust guidelines and good borrow checker practices, even though I might write a lot of ugly, spaghetti code here.

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

Run the program from your favorite console. Here are some useful command line options:

```
 -h | --help       Shows help prompt, then exit.
 --port=XXXX       Uses port XXXX as HTTP port, instead of 3000.
```

Once the program runs, you'll be greeted with a prompt, awaiting input. There, you can use the following commands on the prompt:

```
help                 -- Shows help prompt.
mine                 -- Mines a new block and rewards local node.
mine ID              -- Mines a new block and rewards ID for it.
save                 -- Saves blockchain to blockchain.json.
save FILE            -- Saves blockchain to FILE.
print                -- Dumps blockchain to console as indented JSON.
dump                 -- [TO-DO] Show blockchain statistics.
alias reg ALIAS ADDR -- Registers ALIAS as an alias for identifier ID.
alias show           -- Shows registered aliases.
alias save           -- Saves aliases to aliases.json.
alias save FILE      -- Saves aliases to FILE.
node reg ADDR        -- Registers an address of format https://127.0.0.1:3000 as a node.
node del ADDR        -- [TO-DO] Deletes an address from nodes.
node show            -- [TO-DO] Shows registered nodes.
send VAL DEST        -- [TO-DO] Sends a value VAL from a local identifier to DEST.
send VAL SRC DEST    -- [TO-DO] Sends a value VAL from SRC to DEST.
resolve              -- Scans through all registered nodes and resolves chain conflicts.
wallet new           -- [TO-DO] Creates a new wallet.
wallet load FILE     -- [TO-DO] Loads wallet saved on FILE.
wallet save          -- Saves loaded wallet to wallet.json.
wallet save FILE     -- Saves loaded wallet to FILE.
wallet show          -- Shows addresses of loaded wallet.
wallet balance       -- Processes blockchain and shows balance for currently loaded wallet.
quit/exit            -- Closes program, saving the blockchain and aliases to default files.
```

Please note that the REPL commands above are still subject to change.

## Relevant links
These are the resources I followed to build this (I might have skipped some of them):
- [Learn Blockchains by building one (this article inspired this repository)](https://hackernoon.com/learn-blockchains-by-building-one-117428612f46)
- [Build Your Own Blockchain](http://ecomunsing.com/build-your-own-blockchain)
- [A Blockchain in 200 Lines of Code](https://medium.com/@lhartikk/a-blockchain-in-200-lines-of-code-963cc1cc0e54)
- [Mastering Blockchain, a Packt eBook. (It was free when I got this link!)](https://www.packtpub.com/packt/offers/free-learning)
- [Technical background of version 1 Bitcoin addresses](https://en.bitcoin.it/wiki/Technical_background_of_version_1_Bitcoin_addresses)
- [Ever wonder how bitcoin (and other cryptocurrencies) really work? (video)](https://www.youtube.com/watch?v=bBC-nXj3Ng4)


## License
This program uses the MIT License. Check the file `LICENSE` for details.

## Copyright
(c) 2017 Lucas Vieira
