// This code is largely inspired on article:
// https://hackernoon.com/learn-blockchains-by-building-one-117428612f46

// Blockchain crates and uses
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate crypto;
extern crate uuid;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::thread;
use std::sync::mpsc;
use uuid::Uuid;
use std::time::SystemTime;
use std::fs::File;
use std::io::prelude::*;


// REPL crates and uses
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;


// ----


#[derive(Serialize, Deserialize, Clone, Hash)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: u64,
}

// ----

#[derive(Serialize, Deserialize, Clone, Hash)]
struct Block {
    index: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    proof: u64,
    previous_hash: String,
}

// ----

#[derive(Serialize, Deserialize, Clone)]
struct Blockchain {
    chain: Vec<Block>,
    current_transactions: Vec<Transaction>,
}

impl Blockchain {
    // Blockchain constructor.
    fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: vec![],
            current_transactions: vec![],
        };
        // Create genesis block
        blockchain.new_block(100, Some(String::from("1")));
        blockchain
    }

    // Load blockchain from file
    fn from_file(filename: String) -> Blockchain {
        let mut f = File::open(filename);
        match f {
            Err(_) => {
                println!("Cannot read blockchain file. Creating a new one.");
                Blockchain::new()
            },
            Ok(mut f) => {
                let mut text = String::new();
                match f.read_to_string(&mut text) {
                    Ok(_) => Blockchain::from_str(&text),
                    Err(_) => {
                        println!("Cannor read blockchain file text. Creating a new one.");
                        Blockchain::new()
                    }
                }
            }
        }
    }

    // Load blockchain from string
    fn from_str(string: &String) -> Blockchain {
        let deserialized = serde_json::from_str(string);
        match deserialized {
            Ok(blockchain) => blockchain,
            Err(_)         => {
                println!("Cannot parse blockchain. Creating a new one.");
                Blockchain::new()
            }
        }
    }

    // Saves blockchain to file
    fn to_file(&self, filename: String) {
        let serialized = serde_json::to_string_pretty(&self)
            .expect("Unable to serialize blockchain!"); // TODO: There must be a way not to lose the chain here...
        let mut f = File::create(filename);
        match f {
            Err(_) => println!("Unable to create file!"),
            Ok(mut f) => f.write_all(serialized.as_bytes())
                .expect("Unable to write blockchain to file!"),
        };
    }

    // Creates a new block in the blockchain
    // proof: The proof given by the PoW algorithm
    // previous_hash: Hash of previous block
    // Return: New created block
    fn new_block(&mut self, proof: u64, previous_hash: Option<String>) -> &Block {
        let block = Block {
            index: self.chain.len() as u64 + 1,
            timestamp: time::precise_time_ns(),
            transactions: self.current_transactions.clone(),
            proof: proof,
            previous_hash: match previous_hash {
                Some(hash) => hash,
                None       => Blockchain::hash(self.chain.last().unwrap())
            }
        };
        self.current_transactions.clear();
        self.chain.push(block.clone());
        //println!("Created block {} with hash {}", block.index, Blockchain::hash(&block));
        self.chain.last().unwrap() // We already pushed a block, so it's ok to unwrap here
    }

    // Creates a new transaction to go into the next mined block.
    // sender: Address of sender
    // recipient: Address of recipient
    // amount: Amount of cash
    // Return: Index of block which will hold this transaction
    fn new_transaction(&mut self, sender: String, recipient: String, amount: u64) -> u64 {
        self.current_transactions.push(Transaction {
            sender: sender.clone(),
            recipient: recipient.clone(),
            amount: amount
        });
        
        self.chain.last()
            .expect("Blockchain is empty! Where is the genesis block?")
            .index + 1
    }

    // Creates a SHA-256 hash of a block
    // block: The block
    // Return: hash string
    fn hash(block: &Block) -> String {
        // Correct this:
        // 1. Serialize block to JSON
        let json = serde_json::to_string(&block).unwrap();
        //println!("JSON Block: {}", json);
        // 2. Transform string into [u8]
        let json_b = json.into_bytes();
        // 3. Hash [u8] using crate sha2 https://crates.io/crates/sha2
        let mut hasher = Sha256::new();
        hasher.input(&json_b);
        hasher.result_str()
    }

    // Simple Proof of Work algorithm.
    // Find a number p' such that hash(pp') contains leading 4 zeroes,
    // where p is the previous p'. p is the previous proof, p' is the new
    // proof.
    // last_proof: last computed proof of work
    fn proof_of_work(&self, last_proof: u64) -> u64 {
        let mut proof: u64 = 0;
        while !Blockchain::valid_proof(last_proof, proof) {
            proof += 1;
        }
        proof
    }

    // Validates the proof of work.
    // last_proof: Previous proof of work
    // proof: Current proof of work
    // Return: Whether proof is correct
    fn valid_proof(last_proof: u64, proof: u64) -> bool {
        // I need to check if this is correct later!
        let mut hasher = Sha256::new();
        let guess = last_proof.to_string() + proof.to_string().as_ref();
        hasher.input(&guess.into_bytes());
        let result = hasher.result_str();
        //println!("Hasher result string: {}", result);
        // Now check if first four characters are zeroes
        &result[..4] == "0000"
    }

    // Mines a new block and appends it to the chain.
    // identifier: Identifier for whoever is mining. Will receive a reward.
    fn mine_block(&mut self, identifier: String) {
        let last_block = self.chain.last().unwrap().clone();
        let last_proof = last_block.proof;

        // Mine
        let proof = self.proof_of_work(last_proof);

        // Reward
        self.new_transaction("0".to_owned(), identifier, 1);

        // Forge the new block
        //let previous_hash = Blockchain::hash(&last_block);
        //self.new_block(proof, Some(previous_hash));
        self.new_block(proof, None);
    }

    fn register_node(&mut self) {
    }
}


// ---
#[derive(Serialize, Deserialize)]
struct Node {
    identifier: String,
}

impl Node {
    fn new() -> Node {
        Node {
            identifier: str::replace(Uuid::new_v4().to_string().as_ref(), "-", ""),
            //balance: 0,
        }
    }
}



// ------------------------
// The original tutorial used Flask with a web interface for the operations.
// However, I'll be building a CLI interface instead. A process will run
// on the background and effectively mine and do the transactions and stuff,
// while the client will be responsible to communicate with the background process.

// Therefore, my work here will be:
// 1. Forking the process to the background
// 2. Create a CLI tool -- a REPL of sorts -- to perform common operations:
//    a. Create a new transaction (From, To)
//    b. Mine a new block
//    c. Save the full blockchain
//    d. Print the blockchain to the console
// 3. Manage daemon/repl communications by creating an eval loop on the
//    daemon, which receives messages and performs actions.

// Signal sending enum
enum ReplCommand {
    Help,
    Transaction { from: String, to: String, amount: u64 },
    Mine { miner: String },
    Save { filename: String },
    Print,
    Dump,
    NewNode,
    Alias { alias: String, identifier: String },
    Quit,
}

// Signal receiving enum
enum DaemonResponse {
    
}

// REPL implementation
// Commands:
// - help:
//   Shows help.
// - mine ALIAS/IDENTIFIER: ✓
//   Mines a new block.
// - save FILENAME:
//   Saves blockchain to FILENAME.
// - print: ✓
//   Prints blockchain on console.
// - dump:
//   Shows daemon status
// - node: ✓
//   Manage nodes
//   - new: ✓
//     Creates a new identifier
//   - alias NEWALIAS IDENTIFIER: ✓
//     Creates an ALIAS to IDENTIFIER
// - send:
//   Sends AMOUNT from ALIAS/IDENTIFIER to ALIAS/IDENTIFIER
// - quit/exit: ✓
//   You know what it does.


// ------------------------

// Stopped at Our Blockchain as an API. I'll have to create a repl and a
// message system of sorts...
// The consensus is also missing! We need to implement the consensus.
fn main() {
    // Communication channels
    let (tx, rx) = mpsc::channel(); // REPL to Daemon
    let (ty, ry) = mpsc::channel(); // Daemon to REPL
    //let (tz, rz) = mpsc::channel(); // Daemon to Log

    // Daemon
    let daemon = thread::spawn(move || {
        // Create blockchain
        //let mut blockchain = Blockchain::new();
        let mut blockchain = Blockchain::from_file("blockchain.json".to_owned());

        ty.send(Ok("DAEMON READY".to_owned()));
        
        loop {
            match rx.recv().unwrap() { // TODO: Workaround for this unwrap
                ReplCommand::Quit => {
                    ty.send(Ok("DAEMON QUIT".to_owned()));
                    break
                },
                ReplCommand::Print => {
                    ty.send(Ok(serde_json::to_string_pretty(&blockchain).unwrap()));
                },
                ReplCommand::Transaction { from, to, amount } => {
                    blockchain.new_transaction(from.clone(), to.clone(), amount);
                    ty.send(Ok("TRANSACTION COMPLETED".to_owned())); // TODO: Validate from balance?
                },
                ReplCommand::Mine { miner } => {
                    let now = SystemTime::now();
                    blockchain.mine_block(miner.clone());
                    match now.elapsed() {
                        Ok(elapsed) => {
                            let ans = format!("BLOCK MINED IN {} SECONDS", elapsed.as_secs());
                            ty.send(Ok(ans.clone()))
                        },
                        Err(_) => ty.send(Err("MINING ERROR".to_owned())),
                    };
                },
                _ => {
                    ty.send(Err("DAEMON NOT IMPLEMENTED".to_owned()));
                },
            };
        };

        println!("Saving blockchain...");
        blockchain.to_file("blockchain.json".to_owned());
        println!("Daemon: closed");
    });


    // REPL
    // Node aliases
    // TODO: Put these on our blockchain
    let mut nodes   = HashMap::new();
    let mut aliases = HashMap::new();

    // Await daemon response
    println!("Daemon started: {}", ry.recv().unwrap().unwrap());
    
    // Editor
    let mut rl = Editor::<()>::new();

    // Load REPL history
    /*if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }*/
    
    loop {
        let readline = rl.readline("USER > ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                let atoms = line.split_whitespace()
                    .collect::<Vec<&str>>();

                // Evaluation
                if atoms.len() > 0 {
                    let command = String::from(atoms[0]).to_lowercase();
                    let args = &atoms[1..];

                    match command.as_ref() {
                        "quit" => break,
                        "exit" => break,
                        "node" => {
                            if args.len() < 1 {
                                println!("Please specify what to do with the node.");
                            } else {
                                let arg0 = String::from(args[0]).to_lowercase();
                                match arg0.as_ref() {
                                    "new" => {
                                        // New node
                                        let new_node = Node::new();
                                        let identifier = new_node.identifier.clone();
                                        nodes.insert(new_node.identifier.clone(), new_node);
                                        println!("New node created: {}", identifier);
                                    },
                                    "alias" => {
                                        // Node alias
                                        if args.len() != 3 {
                                            println!("Please specify the node alias and its identifier.");
                                        } else {
                                            let alias = String::from(args[1]);
                                            let identifier = String::from(args[2]);
                                            if nodes.contains_key(&identifier) {
                                                aliases.insert(alias.clone(), identifier.clone());
                                            }
                                            println!("Added alias \"{}\" to identifier {}", alias, identifier);
                                        }
                                    },
                                    _ => println!("Unknown subcommand for \"node\"."),
                                }
                            }
                        },
                        "mine" => {
                            if args.len() != 1 {
                                println!("Please specify the alias or identifier to be rewarded for mining.");
                            } else {
                                let miner = String::from(args[0]);
                                let mut identifier = String::new();
                                match aliases.get(&miner) {
                                    Some(id) => identifier = id.clone(),
                                    None => {
                                        match nodes.get(&miner) {
                                            Some(_) => identifier = miner.clone(),
                                            None => println!("No alias nor registered identifier \"{}\" was found.", miner),
                                        };
                                    },
                                };
                                

                                if identifier.len() > 0 {
                                    println!("Starting block mining process...");
                                    tx.send(ReplCommand::Mine { miner: identifier.clone() });
                                    println!("Awaiting block mining completion...");
                                    match ry.recv().unwrap() {
                                        Ok(status) => println!("Mined block successfully: {}", status),
                                        Err(status) => println!("Block mining error: {}", status),
                                    };
                                }
                            }
                        },
                        "print" => {
                            println!("Requesting blockchain in readable format...");
                            tx.send(ReplCommand::Print);
                            println!("Retrieving response...");
                            println!("Printing blockchain:\n{}", ry.recv().unwrap().unwrap());
                        },
                        //"send" => {},
                        //"save" => {},
                        //"help" => {},
                        _ => println!("Not Implemented"),
                    };
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("C-c");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("C-d");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }

        // TODO: Retrieve 
    }
    //rl.save_history("history.txt").unwrap();

    

    /*tx.send(ReplCommand::Mine { miner: node.identifier.clone() }); // Mine the first block

    // Send a coin to a friend
    tx.send(ReplCommand::Transaction { from:   node.identifier.clone(),
                                       to:     friend.identifier.clone(),
                                       amount: 1 });
    
    // We mine some more blocks
    for _ in 0..7 {
        tx.send(ReplCommand::Mine { miner: node.identifier.clone() });
    }
    
    // Send one more coin to a friend
    tx.send(ReplCommand::Transaction { from:   node.identifier.clone(),
                                       to:     friend.identifier.clone(),
                                       amount: 1 });

    tx.send(ReplCommand::Print);*/
    
    tx.send(ReplCommand::Quit);

    // Also, use ry to receive Daemon feedback.
    
    daemon.join();
}


// ------------------------

// Tests
#[test]
fn proof_of_work() {
    let mut blockchain = Blockchain::new();
    println!("First proof of work: {}",
             blockchain.proof_of_work(blockchain
                                      .chain
                                      .last().unwrap()
                                      .proof));
}

#[test]
fn serialize_deserialize() {
    // Create the blockchain, mine a few blocks, make some
    // transactions, save them by mining one more block
    let mut blockchain = Blockchain::new();
    let node   = Node::new();
    let friend = Node::new();

    // TODO: I might need to register the nodes here soon
    
    for _ in 0..3 {
        blockchain.mine_block(node.identifier.clone());
    }
    blockchain.new_transaction(node.identifier.clone(),
                               friend.identifier.clone(),
                               1);
    blockchain.new_transaction(node.identifier.clone(),
                               friend.identifier.clone(),
                               2);
    blockchain.new_transaction(friend.identifier.clone(),
                               node.identifier.clone(),
                               1);
    blockchain.mine_block(node.identifier.clone());

    // Serialize to string, then reverse it, then serialize
    // the deserialized
    let serialized = serde_json::to_string(&blockchain)
        .expect("Error while serializing blockchain");
    let deserialized: Blockchain = serde_json::from_str(&serialized)
        .expect("Error while deserializing blockchain");
    let reserialized = serde_json::to_string(&deserialized)
        .expect("Error while serializing the new blockchain");

    // Test passes if serialization is equal for both
    println!("First serialization: {}", serialized);
    println!("Second serialization: {}", reserialized);
    
    assert_eq!(serialized, reserialized);
}

// TODO: Add daemon test
