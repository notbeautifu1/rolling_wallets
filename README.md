# rolling_wallets
This simple software allows one to generate private_key for a wallet with user-defined pattern (like 0x0000 and others) and private_key for a wallet, which will create a contract with user-defined pattern

# Setting up .config.json

***num_of_cpus*** defines the number of parallel computing threads. Insert the number of physical cpu cores of your PC. If your PC has a decent CPU and some good cooling system you can insert double the number of cores

***wallets_concat*** here you insert all the wallet patterns, you want to find. _Always remember that pattern MUST start with 0x_

***contracts_concat*** here you insert all the contract patterns, you want to find. _Always remember that pattern MUST start with 0x_

> Just a kind reminder to remember to save the config file after editing

# Running

To run the software you need to use one of two commands: ***contract*** or ***wallet***

### Wallet

***wallet*** command will generate private key for a wallet, which starts with pattern, defined in _wallets_concat_ in _config.json_. The final list is saved in file _wallet.txt_ in the same directory in format _wallet_address:wallet_private_key_. To run software in this mode:

> cargo run --release wallet

### Contract

***contract*** command will generate private key for a wallet, which will create a contract, which starts with a pattern, defined in _contracts_concat_ in _config.json_, at certain nonce. The final list is saved in file _contract.txt_ in the same directory in format _wallet_address:wallet_private_key:contract_address:nonce_. To run software in this mode:

> cargo run --release contract

_Remember that nonce starts with 0 for every wallet, which means that your first transaction will have nonce 0, second tx will have nonce 1 and so on. So if in contract.txt you see nonce number 5 for a particular contract, it means that you need to make 5 outgoing transactions and then deploy the contract. Nonce is only increased on outgoing transactions, i.e. on transactions sent from a particular wallet. Incoming transactions doesn't affect the nonce (for example ETH deposit from CEX). If you need to increase the nonce to reach a particular number, you can just send 0 ETH to your own address, this way you will just burn gas fees_
