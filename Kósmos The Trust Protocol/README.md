Kósmos: The Trust Protocol
Welcome to the official Soroban implementation of Kósmos, a privacy-centric, on-chain reputation and identity network built on Stellar.

This project uses Decentralized Identity (DIDs), Verifiable Credentials (VCs), and Zero-Knowledge Proofs (ZKPs) to establish a flexible, non-financial trust graph.

Vision:
To create a global, non-financial trust layer on the Stellar network that is flexible, user-centric, and entirely privacy-preserving.

Project Structure:
This repository is a Soroban workspace containing the core Kósmos smart contracts:

/contracts/did_registry: The smart contract responsible for managing did:kosmos identifiers. It handles the registration, resolution, and revocation of DID documents.

/contracts/issuer_whitelist: A contract that maintains a dynamic, admin-controlled list of trusted entities (e.g., banks, universities) authorized to issue Verifiable Credentials within the Kósmos ecosystem.

/contracts/zkp_verifier: The core privacy-preserving contract. It executes the cryptographic verification logic for submitted ZKPs, allowing users to prove claims (e.g., "credit score > 700") without revealing the underlying private data.

Getting Started:
Each contract is its own crate within the contracts directory. To build a specific contract (e.g., did_registry):

cd contracts/did_registry
soroban contract build
