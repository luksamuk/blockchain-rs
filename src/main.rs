// This code is largely inspired on article:
// https://hackernoon.com/learn-blockchains-by-building-one-117428612f46

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate time;
extern crate crypto;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::thread;
use std::sync::mpsc;


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
        println!("Created block {} with previous_hash {}", block.index, block.previous_hash);
        println!("Current hash: {}", Blockchain::hash(&block));
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
        println!("JSON Block: {}", json);
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
        let guess = (last_proof * proof).to_string();
        hasher.input(&guess.into_bytes());
        let result = hasher.result_str();
        //println!("Hasher result string: {}", result);
        // Now check if first four characters are zeroes
        &result[..4] == "0000"
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
    Transaction { from: String, to: String, amount: u64 },
    Mine,
    Save { filename: String },
    Print,
    Quit,
}

// Signal receiving enum
enum DaemonResponse {
    
}

// REPL implementation



// ------------------------

// Stopped at Our Blockchain as an API. I'll have to create a repl and a
// message system of sorts...
fn main() {
    // Communication channels
    let (tx, rx) = mpsc::channel(); // REPL to Daemon
    let (ty, ry) = mpsc::channel(); // Daemon to REPL

    // Daemon
    let daemon = thread::spawn(move || {
        println!("Daemon: start");

        // Create blockchain
        // TODO: Populate from existing data
        let mut blockchain = Blockchain::new();
        println!("Daemon: Created blockchain");

        loop {
            match rx.recv().unwrap() { // TODO: Workaround for this unwrap
                ReplCommand::Quit => {
                    ty.send(Ok("DAEMON QUIT"));
                    break
                },
                ReplCommand::Print => {
                    println!("Daemon: Dumping blockchain to console...");
                    println!("Daemon: {}", serde_json::to_string(&blockchain).unwrap());
                    println!("Daemon: Done.");
                    ty.send(Ok("DAEMON PRINT"));
                },
                ReplCommand::Transaction { from, to, amount } => {
                    println!("Daemon: Sending ${} from {} to {}...", amount, from, to);
                    println!("Daemon: Not yet implemented");
                    ty.send(Err("DAEMON NOT IMPLEMENTED"));
                },
                _ => {
                    println!("Daemon: Not yet implemented");
                    ty.send(Err("DAEMON NOT IMPLEMENTED"));
                },
            };
        };

        println!("Daemon: closed");
    });


    // REPL
    // TODO. For now, we're just printing and testing stuff.
    tx.send(ReplCommand::Print);
    tx.send(ReplCommand::Transaction { from:   "SENDER".to_owned(),
                                       to:     "RECIPIENT".to_owned(),
                                       amount: 100 });
    tx.send(ReplCommand::Quit);

    // Also, use ry to receive Daemon feedback.
    

    /*println!("First proof of work: {}",
             blockchain.proof_of_work(blockchain
                                      .chain
                                      .last().unwrap() // ok to unwrap here, I have at least the genesis block
                                      .proof));*/
    daemon.join();
}
