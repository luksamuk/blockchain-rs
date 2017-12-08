// Blockchain crates and uses
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate time;
extern crate crypto;
extern crate uuid;
extern crate rust_base58;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::thread;
use std::sync::mpsc;
use uuid::Uuid;
use std::time::SystemTime;
use std::fs::File;
use rust_base58::{ToBase58, FromBase58};


// REPL crates and uses
extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashSet;
use std::collections::HashMap;


// HTTP server crates and uses
extern crate tiny_http;
extern crate url;
extern crate reqwest;


use tiny_http::Method;
use std::env;
use url::Url;
use std::io::{Write, Read};


// TODO
// node new            => wallet new (???? or maybe even remove)
// node del (also change nodes to hashset. nodes will be just addresses)
// node show


// ----
// Help commands
static HELP_PROMPT: &'static str =
    "help                 -- Shows this prompt.\n\
     mine                 -- Mines a new block and rewards local node.\n\
     mine ID              -- Mines a new block and rewards ID for it.\n\
     save                 -- Saves blockchain to blockchain.json.\n\
     save FILE            -- Saves blockchain to FILE.\n\
     print                -- Dumps blockchain to console as indented JSON.\n\
     dump                 -- [TO-DO] Show blockchain statistics.\n\
     alias reg ALIAS ADDR -- Registers ALIAS as an alias for identifier ID.\n\
     alias show           -- Shows registered aliases.\n\
     alias save           -- Saves aliases to aliases.json.\n\
     alias save FILE      -- Saves aliases to FILE.\n\
     node reg ADDR        -- Registers an address of format https://127.0.0.1:3000 as a node.\n\
     node del ADDR        -- [TO-DO] Deletes an address from nodes.\n\
     node show            -- [TO-DO] Shows registered nodes.\n\
     send VAL DEST        -- [TO-DO] Sends a value VAL from a local identifier to DEST.\n\
     send VAL SRC DEST    -- [TO-DO] Sends a value VAL from SRC to DEST.\n\
     resolve              -- Scans through all registered nodes and resolves chain conflicts.\n\
     wallet new           -- [TO-DO] Creates a new wallet.\n\
     wallet load FILE     -- [TO-DO] Loads wallet saved on FILE.\n\
     wallet save          -- Saves loaded wallet to wallet.json.\n\
     wallet save FILE     -- Saves loaded wallet to FILE.\n\
     wallet show          -- Shows addresses of loaded wallet.\n\
     wallet balance       -- Processes blockchain and shows balance for currently loaded wallet.\n\
     quit/exit            -- Closes program, saving the blockchain and aliases to default files.";

// DEPRECATED:
// node new             -- Generates a new local identifier.\n\

// ----

// Each node is indexed in the blockchain and represents a registered
// node on the network.
// This structure will contain the node data required for transactions.
#[derive(Serialize, Deserialize, Clone)]
struct Node {
    identifier: String,
}

// ----

// Represents a transaction on the blockchain.
#[derive(Serialize, Deserialize, Clone, Hash)]
struct Transaction {
    sender: String,
    recipient: String,
    amount: i64,
}

// ----

// Represents a single block on the blockchain.
#[derive(Serialize, Deserialize, Clone, Hash)]
struct Block {
    index: u64,
    timestamp: u64,
    transactions: Vec<Transaction>,
    proof: u64,
    previous_hash: String,
}

// ----

// Represents the blockchain itself.
#[derive(Serialize, Deserialize, Clone)]
struct Blockchain {
    chain: Vec<Block>,
    current_transactions: Vec<Transaction>,
    nodes: HashSet<String>,
}

impl Blockchain {
    // Blockchain constructor.
    fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain:                vec![],
            current_transactions: vec![],
            nodes:                HashSet::new(),
        };
        // Create genesis block
        blockchain.new_block(100, Some(String::from("1")));
        blockchain
    }

    // Load blockchain from file
    fn from_file(filename: String) -> Blockchain {
        let f = File::open(filename);
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
                        println!("Cannot read blockchain file text. Creating a new one.");
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
        // TODO: There must be a way not to lose the chain here...
        let serialized = serde_json::to_string_pretty(&self)
            .expect("Unable to serialize blockchain!");
        
        let f = File::create(filename);
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
    fn new_transaction(&mut self, sender: String, recipient: String, amount: i64) -> u64 {
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

    // Creates a new unique node identifier.
    // This will not add it to the blockchain.
    fn new_identifier() -> String {
        str::replace(Uuid::new_v4().to_string().as_ref(), "-", "")
    }

    // Determines if a blockchain is valid.
    // chain: Vector of blocks, normally fetched from remote node
    fn valid_chain(chain: &Vec<Block>) -> bool {
        for i in 1..chain.len() {
            // Check if hash of block is correct
            if chain[i].previous_hash != Blockchain::hash(&chain[i - 1]) {
                return false;
            }
            // Check if proof of work is correct
            if !Blockchain::valid_proof(chain[i - 1].proof, chain[i].proof) {
                return false;
            }
        }
        true
    }

    // This is our Consensus Algorithm. It resolves conflicts
    // by replacing our chain with the longest one on the
    // network.
    // Return: Whether our chain was replaced or not.
    fn resolve_conflicts(&mut self) -> bool {
        // We're only looking for chains bigger than ours
        let mut max_length = self.chain.len();
        let mut new_chain: Option<Vec<Block>> = None;

        // Grab and verify the chains from all nodes on the network
        for node in &self.nodes {
            if node != "local" { // TODO: I don't need to check this anymore!
                let proto_uri = format!("{}/chain", node);
                //println!("Fetching chain from {}", proto_uri);

                // Reqwest glue code
                let mut res = reqwest::get(proto_uri.as_ref() as &str).unwrap();

                if res.status().is_success() {
                    let mut body = String::new();
                    let _ = res.read_to_string(&mut body);
                    let chain: Vec<Block> = match serde_json::from_str(&body) {
                        Ok(deserialized) => deserialized,
                        Err(_) => vec![],
                    };
                    //println!("Comparing chain: {}", body);

                    // We're looking only for chains bigger than ours.
                    if chain.len() > max_length && Blockchain::valid_chain(&chain) {
                        new_chain = Some(chain.clone());
                        max_length = chain.len();
                    }
                } else {
                    //println!("Error fetching remote chain: HTTP {}", res.status());
                }
            }
        }
        match new_chain {
            Some(chain) => {
                self.chain = chain.clone();
                true
            },
            None => false
        }
    }

    // EXTRA: Generate a 25-byte binary address from an identifier.
    // Notice that the identifier can be any string here. You should use
    // a pubkey instead of the dumb hashing I use on this example.
    // Also, this is the algorithm described for Bitcoin; i just wanted
    // to have a good-looking address, after all.
    fn generate_address_bin(identifier: &String) -> String {
        // We perform SHA-256 on the pubkey, which is our identifier.
        let sha256step = {
            let mut hasher = Sha256::new();
            hasher.input(&identifier.clone().into_bytes());
            hasher.result_str()
        };
        //println!("First SHA-256 step: {}", sha256step);

        // Perform RIPEMD-160 on the hash
        let ripemd160step = {
            let mut hasher = crypto::ripemd160::Ripemd160::new();
            hasher.input(&sha256step.into_bytes());
            hasher.result_str()
        };

        // Add version byte to front of hash (00 here is Bitcoin's main net,
        // so we'll just do the same)
        let ripemd160step = "00".to_owned() + ripemd160step.as_ref();

        //println!("RIPEMD-160 step plus version: {}", ripemd160step);

        // Perform two SHA256 on it
        let sha256step2n3 = {
            let mut hasher1 = Sha256::new();
            let mut hasher2 = Sha256::new();
            hasher1.input(&ripemd160step.clone().into_bytes());
            hasher2.input(&hasher1.result_str().into_bytes());
            hasher2.result_str()
        };
        //println!("SHA-256 steps 2 and 3: {}", sha256step2n3);

        // Get address checksum; first 4 bytes (8 characters) of last step
        let checksum = String::from(&sha256step2n3[..8]);
        //println!("Checksum: {}", checksum);

        // Add checksum to extended RIPEMD-160 address, generating our
        // 25-byte binary address.
        // So this is already our binary address.
        checksum + ripemd160step.as_ref()
    }

    // EXTRA: Generate a cute address from our binary address.
    fn generate_address(bin_addr: &String) -> String {
        assert_eq!(bin_addr.len(), 50);
        // We need to gen a vec of numbers from string pairs of letters,
        // then convert to base 58.
        let mut binvec = vec![];

        for i in 0..25 {
            let hex = &bin_addr[i*2..(i*2)+2];
            binvec.push(i64::from_str_radix(hex, 16).unwrap() as u8);
        }

        binvec.to_base58()
    }

    fn generate_binaddr_from(address: &String) -> String {
        let binvec2: Vec<_>  = address.from_base58().unwrap();
        let binaddr2str: Vec<String> = binvec2.iter()
            .map(|b| format!("{:02x}", b))
            .collect();
        binaddr2str.join("")
    }
}

// ---

// Represents a wallet.
// By default, we'll only use a single wallet.
#[derive(Serialize, Deserialize, Clone)]
struct Wallet {
    addresses: Vec<String>,
    balances:  Vec<i64>,
    last_block_checked: usize,
}

impl Wallet {
    fn new() -> Wallet {
        let mut wallet = Wallet { addresses: vec![], balances: vec![], last_block_checked: 1, };
        // We'll be using only 5 addresses per wallet, we don't need more for now.
        for _ in 0..5 {
            let identifier = Blockchain::new_identifier();
            let bin_addr = Blockchain::generate_address_bin(&identifier);
            wallet.addresses.push(Blockchain::generate_address(&bin_addr));
            wallet.balances.push(0);
        }
        wallet
    }

    

    fn calculate_balances(&mut self, chain: &Vec<Block>) {
        // If we have more blocks than when last checked, calculate
        let last_chain_idx = chain.last().unwrap().index as usize;
        if last_chain_idx > self.last_block_checked {
            for i in self.last_block_checked .. last_chain_idx {
                let block = &chain[i];
                let mut n = 0;
                for addr in &self.addresses {
                    for transaction in &block.transactions {
                        if transaction.recipient == *addr {
                            self.balances[n] += transaction.amount;
                        } else if transaction.sender == *addr {
                            self.balances[n] -= transaction.amount;
                        }
                    }
                    n += 1;
                }
            }
            self.last_block_checked = last_chain_idx;
        }
    }

    // Load wallet from file
    fn from_file(filename: String) -> Wallet {
        let f = File::open(filename);
        match f {
            Err(_) => {
                println!("Cannot read wallet file. Creating a new one.");
                Wallet::new()
            },
            Ok(mut f) => {
                let mut text = String::new();
                match f.read_to_string(&mut text) {
                    Ok(_) => Wallet::from_str(&text),
                    Err(_) => {
                        println!("Cannot read wallet file text. Creating a new one.");
                        Wallet::new()
                    }
                }
            }
        }
    }

    // Load wallet from string
    fn from_str(string: &String) -> Wallet {
        let deserialized = serde_json::from_str(string);
        match deserialized {
            Ok(wallet) => wallet,
            Err(_)         => {
                println!("Cannot parse wallet. Creating a new one.");
                Wallet::new()
            }
        }
    }

    // Save wallet to file
    fn to_file(&self, filename: String) {
        let serialized = serde_json::to_string_pretty(self)
            .expect("Unable to serialize wallet!");
        let f = File::create(filename);
        match f {
            Err(_) => println!("Unable to create file!"),
            Ok(mut f) => f.write_all(serialized.as_bytes())
                .expect("Unable to write wallet to file!"),
        };
    }
}


// ---

// ------------------------
// The original tutorial used Flask with a web interface for the operations.
// However, I'll be building a CLI interface instead. A process will run
// on the background and effectively mine and do the transactions, consensus, etc,
// while the client will be responsible to communicate with the background process.

// Signal sending enum
enum ReplCommand {
    Transaction { from: String, to: String, amount: i64 },
    Mine { miner: String },
    Save { filename: String },
    Print,
    Dump,
    RegNode { url: String },
    //CheckNode { identifier: String },
    GetChain,
    Resolve,
    Quit,

    HttpGetChain,
}


// ------------------------

// Loads aliases from an aliases file, then deserializes it to
// a HashMap.
fn load_aliases(filename: String) -> HashMap<String, String> {
    let f = File::open(filename);
        match f {
            Err(_) => {
                println!("Cannot read aliases file.");
                HashMap::new()
            },
            Ok(mut f) => {
                let mut text = String::new();
                match f.read_to_string(&mut text) {
                    Ok(_) => {
                        let deserialized = serde_json::from_str(text.as_ref());
                        match deserialized {
                            Ok(aliases) => aliases,
                            Err(_)      => {
                                println!("Cannot parse aliases.");
                                HashMap::new()
                            }
                        }
                    },
                    Err(_) => {
                        println!("Cannot read alias file text.");
                        HashMap::new()
                    }
                }
            }
        }
}

// Serializes aliases to a JSON file.
fn save_aliases(aliases: &HashMap<String, String>, filename: String) {
    let serialized = serde_json::to_string_pretty(aliases)
        .expect("Unable to serialize aliases!");
    let f = File::create(filename);
    match f {
        Err(_) => println!("Unable to create file!"),
        Ok(mut f) => f.write_all(serialized.as_bytes())
            .expect("Unable to write aliases to file!"),
    };
}

// ------------------------

// Stopped at Our Blockchain as an API. I'll have to create a repl and a
// message system of sorts...
// The consensus is also missing! We need to implement the consensus.
fn main() {
    let mut node_port = "3000".to_owned(); // Default HTTP service port

    println!("blockchain-rs 0.5.0");
    println!("Copyright (C) 2017 Lucas Vieira.");
    println!("This program is distributed under the MIT License. Check source code for details.");
    
    /* ===== Console Arguments  ===== */
    let args: Vec<_> = env::args().collect();
    for argument in args {
        let midplace = argument.find('=');
        match midplace {
            None => {
                // Not a configuration, what a shame.
                // Check for a --help or -h here later.
                match argument.as_ref() {
                    "-h" | "--help" => {
                        println!("Command line options:");
                        println!(" -h | --help       Shows help prompt, then exit.");
                        println!(" --port=XXXX       Uses port XXXX as HTTP port, instead of 3000.");
                        println!("\nREPL commands:\n{}", HELP_PROMPT);
                        return;
                    },
                    _ => {
                        // Unknown command
                    },
                }
            },
            Some(position) => {
                let argname = &argument[..position];
                let argcfg  = &argument[position + 1..];
                match argname {
                    "--port" => {
                        node_port = argcfg.to_owned();
                    },
                    _ => {},
                }
            }
        }
    }

    // Communication channels
    let (tx, rx) = mpsc::channel();        // REPL to Daemon
    let (ty, ry) = mpsc::channel();        // Daemon to REPL
    let (tz, rz) = mpsc::channel();        // Daemon to HTTP service
    let txhttp = tx.clone();               // HTTP service to Daemon using REPL commands

    

    /* ===== DAEMON ===== */
    let daemon = thread::spawn(move || {
        // Create blockchain
        let mut blockchain = Blockchain::from_file("blockchain.json".to_owned());

        let _ = ty.send(Ok("DAEMON READY".to_owned()));
        
        loop {
            match rx.recv().unwrap() { // TODO: Workaround for this unwrap
                ReplCommand::Quit => {
                    let _ = ty.send(Ok("DAEMON QUIT".to_owned()));
                    break
                },
                ReplCommand::Print => {
                    let _ = ty.send(Ok(serde_json::to_string_pretty(&blockchain).unwrap()));
                },
                ReplCommand::Transaction { from, to, amount } => {
                    blockchain.new_transaction(from.clone(), to.clone(), amount);
                    let _ = ty.send(Ok("TRANSACTION COMPLETED".to_owned())); // TODO: Validate from balance?
                },
                ReplCommand::Mine { miner } => {
                    let now = SystemTime::now();
                    blockchain.mine_block(miner.clone());
                    let _ = match now.elapsed() {
                        Ok(elapsed) => {
                            let ans = format!("BLOCK MINED IN {} SECONDS", elapsed.as_secs());
                            ty.send(Ok(ans.clone()))
                        },
                        Err(_) => ty.send(Err("MINING ERROR".to_owned())),
                    };
                },
                ReplCommand::Save { filename } => {
                    blockchain.to_file(filename.clone());
                    let _ = ty.send(Ok("FILE SAVED".to_owned()));
                },
                ReplCommand::RegNode { url } => {
                    blockchain.nodes.insert(url.clone());
                    let _ = ty.send(Ok("REGISTERED".to_owned()));
                },
                // TODO: Change this to verify if it isn't going to make us crash?
                /*ReplCommand::CheckNode { identifier } => {
                    let mut exists = false;
                    for (_, node) in &blockchain.nodes {
                        if node.identifier == identifier {
                            exists = true;
                            break;
                        }
                    }
                    if exists {
                        let _ = ty.send(Ok("NODE EXISTS".to_owned()));
                    } else {
                        let _ = ty.send(Err("NODE DOESN'T EXIST".to_owned()));
                    }
                },*/
                ReplCommand::Resolve => {
                    let changed = blockchain.resolve_conflicts();
                    let _ = match changed {
                        true  => ty.send(Ok("CHAIN UPDATED".to_owned())),
                        false => ty.send(Ok("CHAIN UP-TO-DATE".to_owned())),
                    };
                },
                ReplCommand::GetChain => {
                    let chain_serialized: String = serde_json::to_string(&blockchain.chain).unwrap();
                    let _ = ty.send(Ok(chain_serialized.clone()));
                },
                ReplCommand::HttpGetChain => {
                    let chain_serialized: String = serde_json::to_string(&blockchain.chain).unwrap();
                    let _ = tz.send(chain_serialized.clone());
                },
                _ => {
                    let _ = ty.send(Err("DAEMON NOT IMPLEMENTED".to_owned()));
                },
            };
        };

        // TODO: uncomment this for automatic blockchain saving!
        println!("Saving blockchain...");
        blockchain.to_file("blockchain.json".to_owned());
        println!("Daemon: closed");
    });



    
    /* ===== HTTP SERVER ===== */
    // I use tiny_http here instead of hyper, since I wanted the server to spawn/run from
    // another thread, and also to have access to some variables I declared above and I
    // borrow inside of it. Hyper is a lot more robust, but would need some advanced tricks
    // so I could use the variables above, specially the channels, so I'm just cutting the
    // crap here and doing a lightweight solution.
    // Clients, though, are going to use hyper, since I faced some problems with reqwest.
    println!("Starting server on port {}", node_port);
    let server = thread::spawn(move || {
        let server = tiny_http::Server::http(format!("127.0.0.1:{}", node_port)).unwrap();
        loop {
            match server.recv() {
                Ok(req) => {
                    if req.method() == &Method::Get && req.url() == "/chain" { // curl -X GET "http://127.0.0.1:3000/chain"
                        let _ = txhttp.send(ReplCommand::HttpGetChain);
                        let response = tiny_http::Response::from_string(rz.recv().unwrap());
                        let _ = req.respond(response);
                    } else {
                        let _ = req.respond(tiny_http::Response::empty(404));
                    }
                },
                Err(_) => {
                    // Something here shouldn't have happened. Hmmm.
                    // I don't care about it, though, unless it kills my server.
                    panic!("Something terribly wrong has happened with the HTTP server. Please restart.");
                },
            }
        }
    }); // Unfortunately, I'm doing something wrong with this: I don't check for thread
    // finishing. I know this is bad, but I still need to figure out a way to make this
    // thread die. It could be forcibly, though, since it isn't running any crucial
    // operations.


    

    /* ===== REPL ===== */
    // Node aliases
    #[derive(Serialize, Deserialize)]
    let mut aliases = load_aliases("aliases.json".to_owned());
    let mut wallet  = Wallet::from_file("wallet.json".to_owned());

    // Await daemon response
    println!("Daemon started: {}", ry.recv().unwrap().unwrap());
    
    // Editor
    let mut rl = Editor::<()>::new();

    // Load REPL history
    /*if let Err(_) = rl.load_history("history.txt") {
        println!("No previous history.");
    }*/

    // REPL's loop
    println!("For a list of commands, type `help`.");
    loop {
        let readline = rl.readline("BLOCKCHAIN > ");
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
                        "quit" | "exit" => break,
                        "node" => {
                            if args.len() < 1 {
                                println!("Please specify what to do with the node.");
                            } else {
                                let arg0 = String::from(args[0]).to_lowercase();
                                match arg0.as_ref() {
                                    /*"new" => {
                                        // 1. Request node creation from blockchain
                                        let _ = tx.send(ReplCommand::NewNode);
                                        // 2. Return identifier
                                        println!("New node created: {}", ry.recv().unwrap().unwrap());
                                    },*/
                                    "reg" => {
                                        if args.len() != 2 {
                                            println!("Please specify an address for the node.");
                                        } else {
                                            let url = args[1].to_owned();
                                            match Url::parse(url.as_ref()) {
                                                Ok(_) => {
                                                    let _ = tx.send(ReplCommand::RegNode { url: url.clone() });
                                                    let _ = ry.recv().unwrap();
                                                    println!("Node registered successfully.");
                                                },
                                                Err(_) => {
                                                    println!("Please provide a valid URL.");
                                                },
                                            }
                                        }
                                    },
                                    "show" => {
                                        // Show registered nodes. Ask daemon to do that.
                                    },
                                    _ => println!("Unknown subcommand for \"node\"."),
                                }
                            }
                        },
                        "mine" => {
                            let mut identifier = String::new();
                            if args.len() != 1 {
                                println!("Assuming mining operation for current wallet's Address #0.");
                                identifier = wallet.addresses[0].clone();
                            } else {
                                let miner = String::from(args[0]);
                                identifier = 
                                    match aliases.get(&miner) {
                                        Some(id) => id.clone(),
                                        None => {
                                            miner.clone()
                                        },
                                    };
                            }
                                

                            if identifier.len() > 0 {
                                println!("Starting block mining process...");
                                let _ = tx.send(ReplCommand::Mine { miner: identifier.clone() });
                                println!("Awaiting block mining completion...");
                                match ry.recv().unwrap() {
                                    Ok(status) => println!("Mined block successfully: {}", status),
                                    Err(status) => println!("Block mining error: {}", status),
                                };
                            }
                            
                        },
                        "print" => {
                            println!("Requesting blockchain in readable format...");
                            let _ = tx.send(ReplCommand::Print);
                            println!("Retrieving response...");
                            println!("Printing blockchain:\n{}", ry.recv().unwrap().unwrap());
                        },
                        "resolve" => {
                            println!("Resolving conflicts between this node and others...");
                            let _ = tx.send(ReplCommand::Resolve);
                            println!("Resolving finished. Daemon response: {}", ry.recv().unwrap().unwrap());
                        },
                        //"send" => {},
                        "save" => {
                            let mut filename = None;
                            if args.len() > 1 {
                                println!("Please specify only one filename.");
                            } else if args.len() == 1 {
                                filename = Some(String::from(args[0]));
                            } else {
                                filename = Some(String::from("blockchain.json"));
                            }

                            match filename {
                                Some(file) => {
                                    let _ = tx.send(ReplCommand::Save { filename: file.clone() });
                                    println!("Blockchain saving to {}: {}", file, ry.recv().unwrap().unwrap());
                                },
                                _ => {}
                            };
                        },
                        "alias" => {
                            // Address alias
                            if args.len() > 0 {
                                let arg0 = String::from(args[0]).to_lowercase();
                                if args.len() > 1 {
                                    match arg0.as_ref() {
                                        "save" => {
                                            let mut filename = None;
                                            if args.len() > 2 {
                                                println!("Please specify only one filename.");
                                            } else if args.len() == 2 {
                                                filename = Some(String::from(args[1]));
                                            } else if args.len() < 2 {
                                                filename = Some(String::from("aliases.json"));
                                            }

                                            match filename {
                                                Some(file) => {
                                                    save_aliases(&aliases, file.clone());
                                                    println!("Aliases saved to {}", file);
                                                },
                                                _ => {},
                                            };
                                        },
                                        "show" => {
                                            for (alias, identifier) in &aliases {
                                                println!("{}: {}", alias, identifier);
                                            }
                                        },
                                        "reg" => {
                                            if args.len() != 3 {
                                                println!("Please specify the alias and its address.");
                                            } else { // alias register ALIAS ADDRESS
                                                let alias = String::from(args[1]);
                                                let addr = String::from(args[2]);

                                                // TODO: Verify whether addr is a true base58 address

                                                aliases.insert(alias.clone(), addr.clone());
                                                println!("Added alias \"{}\" to address {}", alias, addr);
                                            }
                                        },
                                        _ => println!("Unknown command for \"alias\"."),
                                    }
                                }
                            } else {
                                println!("Please specify what operation to perform with aliases.");
                            }
                        },
                        "wallet" => {
                            if args.len() < 1 {
                                println!("Please specify what to do with the current wallet.");
                            } else {
                                let arg0 = String::from(args[0]).to_lowercase();
                                match arg0.as_ref() {
                                    //"new" => {},
                                    //"load" => {},
                                    "show" => {
                                        println!("Addresses:");
                                        for addr in &wallet.addresses {
                                            println!("{}", addr);
                                        }
                                        println!("\nTotal balance: ${}.\nThere may be unconfirmed transactions, run `wallet balance` to update.",
                                                 wallet.balances
                                                 .iter()
                                                 .fold(0, |acc, &x| acc + x));
                                    },
                                    "save" => {
                                        let mut filename = String::new();
                                        if args.len() < 2 {
                                            filename = "wallet.json".to_owned();
                                        } else if args.len() >= 2 {
                                            filename = String::from(args[1]);
                                        }
                                        wallet.to_file(filename.clone());
                                        println!("Saved wallet to {}.", filename);
                                    },
                                    "balance" => {
                                        // Request chain from daemon
                                        println!("Requesting blockchain from local daemon...");
                                        let _ = tx.send(ReplCommand::GetChain);
                                        let chain_serialized = ry.recv().unwrap().unwrap();
                                        let chain: Vec<Block> = serde_json::from_str(&chain_serialized).unwrap();
                                        
                                        println!("Updating balance...");
                                        wallet.calculate_balances(&chain);

                                        println!("Updated balance: ${}",
                                                 wallet.balances.iter().fold(0, |acc, &x| acc + x));
                                    },
                                    _ => println!("Unknown subcommand for \"wallet\"."),
                                }
                            }
                        },
                        "help" => {
                            println!("Useful commands:\n{}", HELP_PROMPT);
                        },
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
    
    let _ = tx.send(ReplCommand::Quit);

    println!("Saving aliases...");
    save_aliases(&aliases, "aliases.json".to_owned());
    println!("Saving wallet...");
    wallet.to_file("wallet.json".to_owned());
    let _ = daemon.join();
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
    let node   = Blockchain::new_identifier();
    let friend = Blockchain::new_identifier();
    
    for _ in 0..3 {
        blockchain.mine_block(node.clone());
    }
    blockchain.new_transaction(node.clone(),
                               friend.clone(),
                               1);
    blockchain.new_transaction(node.clone(),
                               friend.clone(),
                               2);
    blockchain.new_transaction(friend.clone(),
                               node.clone(),
                               1);
    blockchain.mine_block(node.clone());

    // Serialize to string, then reverse it, then serialize
    // the deserialized
    let serialized = serde_json::to_string(&blockchain)
        .expect("Error while serializing blockchain");
    let deserialized: Blockchain = serde_json::from_str(&serialized)
        .expect("Error while deserializing blockchain");
    let reserialized = serde_json::to_string(&deserialized)
        .expect("Error while serializing the new blockchain");

    // Test passes if serialization is equal for both
    //println!("First serialization: {}", serialized);
    //println!("Second serialization: {}", reserialized);
    
    assert_eq!(serialized, reserialized);
}

#[test]
fn address_gen() {
    // Let's pretend we're creating a public key.
    // JUST SO YOU KNOW! THIS IS NOTHING MORE THAN
    // AN IDENTIFIER!
    // I say it's a pubkey here, but it really isn't.
    // I won't be implementing
    let pubkey = Blockchain::new_identifier();
    println!("Pubkey: {}", pubkey);

    // Generate a cute 25-byte binary address.
    let binaddr = Blockchain::generate_address_bin(&pubkey);
    println!("Binary address #1: {}", binaddr);

    // Now generate a cute address for a fictional wallet.
    let address = Blockchain::generate_address(&binaddr);
    println!("Address: {}", address);
    
    // Return generated address to 25-byte address so we
    // verify the algorithm's integrity
    let binaddr2 = Blockchain::generate_binaddr_from(&address);
    println!("Binary address #2: {}", binaddr2);
    
    assert_eq!(binaddr, binaddr2);
}

#[test]
fn wallet_gen() {
    // We generate a wallet and test it.
    let mut wallet = Wallet::new();
    println!("Generated wallet");

    // We create a blockchain
    let mut blockchain = Blockchain::new();
    
    // Mine seven blocks for first address
    for i in 0..7 {
        println!("Mining block #{}, rewarding $1 to address #1", i + 1);
        blockchain.mine_block(wallet.addresses[0].clone());
    }

    // First address is generous and will give 1 currency to other addresses
    for i in 1..wallet.addresses.len() {
        println!("Address #1 will send 1 currency to address #{}", i + 1);
        blockchain.new_transaction(wallet.addresses[0].clone(),
                                   wallet.addresses[i].clone(),
                                   1);
    }

    // First address is specially fond of address #3 and will give it two more
    println!("Address #1 will send 1 currency to address #3");
    blockchain.new_transaction(wallet.addresses[0].clone(),
                               wallet.addresses[2].clone(),
                               1);

    // Mine a new block to confirm changes
    println!("Mining new block for #1 to confirm previous transactions...");
    blockchain.mine_block(wallet.addresses[0].clone());

    // Checking balance...
    wallet.calculate_balances(&blockchain.chain);

    // We expect:
    // Address #1 will have $2 (+ one unconfirmed mining bounty, which will not be shown)
    // Address #3 will have $2
    // Everyone else gets $1
    assert_eq!(wallet.balances[0], 2);
    assert_eq!(wallet.balances[1], 1);
    assert_eq!(wallet.balances[2], 2);
    assert_eq!(wallet.balances[3], 1);
    assert_eq!(wallet.balances[4], 1);

    // TODO: Check balance again after one more round

    for i in 0..wallet.addresses.len() {
        println!("Address #{}: {}, Balance: ${}", i + 1, wallet.addresses[i], wallet.balances[i]);
    }
    println!("Unconfirmed transactions:");
    for transaction in &blockchain.current_transactions {
        println!("Sender: {}, Recipient: {}, Amount: ${}",
                 transaction.sender, transaction.recipient, transaction.amount);
    }
}
