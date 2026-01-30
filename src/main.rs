mod engine;
mod types;

use std::env;
use std::process;

use engine::Engine;
use types::{AccountOutput, TransactionRecord};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <transactions.csv>", args[0]);
        process::exit(1);
    }

    let mut engine = Engine::new();

    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(&args[1])
        .unwrap_or_else(|e| {
            eprintln!("Error opening {}: {}", args[1], e);
            process::exit(1);
        });

    for result in reader.deserialize::<TransactionRecord>() {
        match result {
            Ok(record) => engine.process(record),
            Err(e) => eprintln!("Skipping malformed row: {}", e),
        }
    }

    let mut writer = csv::Writer::from_writer(std::io::stdout());
    for account in engine.output() {
        if let Err(e) = writer.serialize::<AccountOutput>(account) {
            eprintln!("Error writing output: {}", e);
        }
    }
}
