**Project Overview**
- A standalone, dependency-free educational cryptocurrency implemented in Rust. The project reproduces many core concepts found in Bitcoin-like systems (transactions, UTXO, merkle trees, mining, signing/verification), while also providing RSA and ECDSA implementations and a custom big-integer library.

**What’s implemented**
- **Cryptography:** RSA and ECDSA (secp256k1) signing and verification are implemented in `src/rsa` and `src/ecdsa`.
- **Hashing:** SHA-256 implementation in `src/sha256` used for transaction and block IDs.
- **Big integers & arithmetic:** Custom `BigInt` implementation and modular arithmetic (Barrett reduction, modular inverse, modular exponentiation, etc.) live in `src/math` and are used across the crypto code.
- **Blockchain primitives:** UTXO model, `Transaction`/`TxInput`/`TxOutput`, `MerkleTree`, `Block`, block mining (proof-of-work) and chain verification are implemented in `src/blockchain`.
- **Node & wallet/user models:** `Node` and `User` abstractions in `src/node` and `src/user` with helper functions for creating transactions, signing inputs, mining blocks, and querying funds.
- **Tests:** Unit tests cover crypto primitives and higher-level behaviours (see `cargo test`).

**Repository layout (quick reference)**
- `src/lib.rs` — public module exports.
- `src/blockchain/` — `block.rs`, `transaction.rs`, `merkle.rs`, `mod.rs`.
- `src/ecdsa/` — elliptical curve and ECDSA utilities (`point.rs`, `secp256k1.rs`).
- `src/rsa/` — RSA keys, encryption, decryption, signing, verification.
- `src/math/` — `big_int.rs`, `algorithms.rs`, `random.rs`.
- `src/sha256/` — SHA-256 implementation.
- `src/node/`, `src/user/` — node and user logic and tests.

**High-level architecture & data flow**
- Transactions: `Transaction` objects carry inputs (references to previous transaction outputs by `txid` + `vout`) and outputs (`TxOutput` with `value` and `script_pubkey`).
- UTXO set: `Blockchain` maintains a `HashMap<Sha256, Vec<TxOutput>>` representing unspent outputs used to validate new transactions.
- Signing: Each input is signed separately. The signing process uses `Transaction::serialize_for_input` to build the message hashed with SHA-256; signatures are produced with `ecdsa::sign` and verified with `ecdsa::verify`.
- Mining: `Block::mine` loops, updating `nonce` and `timestamp` until block hash meets `Sha256::is_valid(difficulty)`.
- Verification: `Blockchain::verify_new_transaction` and `Blockchain::verify_new_block` enforce signature correctness, UTXO availability, merkle root integrity, difficulty target, and coinbase rules.

**API summary (important types & functions)**
- `blockchain::Blockchain` — manage blocks and UTXO, verify and append blocks: `new`, `create_block`, `add_block`, `verify_new_transaction`, `verify_chain`, `get_user_funds`.
- `blockchain::Block` — block data structure and mining: `new`, `new_genesis`, `mine`, `hash`.
- `blockchain::transaction::Transaction` — create coinbase (`get_coinbase`), create inputs/outputs, compute transaction hash and input-specific hashes for signing.
- `ecdsa` — `generate_keypair`, `sign(message, &ECDSAPrivateKey)` → signature `AffinePoint`, `verify(signature, message, &ECDSAPublicKey)` → `bool`.
- `rsa` — `generate_keys`, `encrypt`, `decrypt`, `sign`, `verify` (implemented for educational comparison).
- `user::User` — helpers for wallet behaviour: `try_transaction`, `update_funds`, `update_funds_from_chain`, `get_funds`, `verify_transaction_prescence`.
- `node::Node` — local node: `add_transaction`, `remove_transaction`, `mine`, `accept_block`, `get_verifiyng_transaction_branch`.