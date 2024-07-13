use anyhow::Result;
use ethers::{
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use ethers_core::rand::thread_rng;
use kanal::unbounded_async;
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::config::load_config;

pub async fn contract() -> Result<()> {
    // Opening the output file, creating it if it doesn't exist
    // Wrapping in Arc<Mutex<>> to safely share it between threads
    let file = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("./contract.txt")?,
    ));

    // Setting up the ctrl+c handler
    ctrlc::set_handler({
        // Creating file clone to safely transmit it to closure
        let file_clone = file.clone();

        move || {
            info!("Received Ctrl+C! Preparing to exit...");

            // Locking the file and flushing the buffer
            if let Ok(mut output_file) = file_clone.lock() {
                output_file
                    .flush()
                    .expect("Error while trying to final flush output file");
            }

            info!("Output file saved successfully!");

            // Terminating the application
            std::process::exit(0);
        }
    })?;

    // Loading config file from ./config.json
    let config = load_config()?;

    // Creating channel to transfer final string (wallet_address:wallet_private_key:contract_address:nonce)
    // This is essential to solve possible async thread issues
    let (transfer_sender, transfer_receiver) = unbounded_async::<String>();

    // Creating predefined in config number of threads for computation
    // NOTE: num_of_cpus in config.json should be pc_cpus, pc_cpus * 2 is only recommended for pcs with decent cooling
    for _ in 0..config.num_of_cpus {
        // Spawning a thread
        tokio::spawn({
            // Cloning final string sender
            let sender = transfer_sender.clone();

            // Cloning vector, containing the searchable contracts
            let searchable_contracts = config.contracts_concat.clone();

            async move {
                loop {
                    // Generating new wallet
                    let wallet = LocalWallet::new(&mut thread_rng());

                    // Calculating contract address for first 10 nonces
                    for index in 0..10 {
                        let nonce: U256 = U256::from(index);

                        // Creating RLP stream and appending wallet address and nonce
                        let mut stream = ethers::utils::rlp::RlpStream::new_list(2);
                        stream.append(&wallet.address().as_bytes());
                        stream.append(&nonce);
                        let rlp_encoded = stream.out();

                        // Calculating hash from RLP stream
                        let hash = ethers::utils::keccak256(&rlp_encoded);

                        // Getting final contract address from the hash
                        let contract_address: String =
                            format!("{:?}", Address::from_slice(&hash[12..32]));

                        for starting in searchable_contracts.iter() {
                            // Comparing if contract address starts with searchable pattern
                            if contract_address.starts_with(starting) {
                                // Saving wallet address into variable
                                let wallet_address = format!("{:?}", wallet.address());

                                // Transforming signer into private_key
                                let private_key = format!(
                                    "0x{}",
                                    wallet
                                        .signer()
                                        .to_bytes()
                                        .iter()
                                        .map(|&i| format!("{:02x}", i))
                                        .collect::<Vec<String>>()
                                        .join("")
                                );

                                // Sending the final string to main thread
                                let _ = sender
                                    .send(format!(
                                        "{}:{}:{}:{}",
                                        wallet_address, private_key, contract_address, nonce
                                    ))
                                    .await;
                            }
                        }
                    }
                }
            }
        });
    }

    // Waiting for final_strings from different threads and then saving them in contract.txt
    while let Ok(final_string) = transfer_receiver.recv().await {
        // Logging
        info!("{}", final_string);

        // Locking the file and flushing the final string
        if let Ok(mut output_file) = file.lock() {
            writeln!(output_file, "{}", final_string)?;
            output_file
                .flush()
                .expect("Error while trying to save into output file");
        }
    }

    Ok(())
}
