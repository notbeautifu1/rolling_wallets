use anyhow::Result;
use ethers::signers::{LocalWallet, Signer};
use ethers_core::rand::thread_rng;
use kanal::unbounded_async;
use log::info;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::config::load_config;

pub async fn wallet() -> Result<()> {
    // Opening the output file, creating it if it doesn't exist
    // Wrapping in Arc<Mutex<>> to safely share it between threads
    let file = Arc::new(Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open("./wallet.txt")?,
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

    // Creating channel to transfer final string (wallet_address:wallet_private_key)
    // This is essential to solve possible async thread issues
    let (transfer_sender, transfer_receiver) = unbounded_async::<String>();

    // Creating predefined in config number of threads for computation
    // NOTE: num_of_cpus in config.json should be pc_cpus, pc_cpus * 2 is only recommended for pcs with decent cooling
    for _ in 0..config.num_of_cpus {
        // Spawning a thread
        tokio::spawn({
            // Cloning final string sender
            let sender = transfer_sender.clone();

            // Cloning vector, containing the searchable wallets
            let searchable_wallets = config.wallets_concat.clone();

            async move {
                loop {
                    // Generating new wallet
                    let wallet = LocalWallet::new(&mut thread_rng());

                    // Saving wallet address into variable
                    let wallet_address = format!("{:?}", wallet.address());

                    for starting in searchable_wallets.iter() {
                        // Comparing if wallet address starts with searchable pattern
                        if wallet_address.starts_with(starting) {
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
                                .send(format!("{}:{}", wallet_address, private_key))
                                .await;
                        }
                    }
                }
            }
        });
    }

    // Waiting for final_strings from different threads and then saving them in wallet.txt
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
