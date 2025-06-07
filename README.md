# IC Multisig Voting

A Rust library and demo for implementing multisignature voting mechanisms on the Internet Computer (IC).

## Project Structure

```
ic-multisig-voting/
├── crates/
│   ├── ic-multisig-voting/     # Core library crate
│   └── ic-multisig-demo/       # Demo canister implementation
├── Cargo.toml                  # Workspace configuration
└── README.md
```

## Overview

This project provides a generic multisig voting system where multiple owners can propose and vote on actions. Once a proposal reaches the required threshold of approvals, it automatically executes.

### Key Features

- **Generic payload support**: Any Candid-serializable type can be proposed
- **Configurable threshold**: Set minimum number of approvals required
- **Auto-execution**: Proposals execute immediately when threshold is met
- **Re-entrancy protection**: Prevents double execution of proposals
- **Owner management**: Only designated owners can propose and vote

## Quick Start

### Prerequisites

- [DFX](https://internetcomputer.org/docs/current/developer-docs/setup/install/) 0.23.0+
- Rust with `wasm32-unknown-unknown` target

### Building and Deployment

1. **Clone and build**:
   ```bash
   git clone <your-repo>
   cd ic-multisig-voting
   cargo build --target wasm32-unknown-unknown --release
   ```

2. **Deploy the demo canister**:
   ```bash
   cd crates/ic-multisig-demo
   dfx start --clean --background
   dfx deploy --argument '(vec {principal "'$(dfx identity get-principal)'"}, 1)'
   ```

3. **Interact with the canister**:
   ```bash
   # Propose a configuration change
   dfx canister call demo propose_set_config '(record { 
     max_payload_size = 2048; 
     allowed_origins = vec {"https://ic0.app"}; 
     admin_fee = 100 
   })'
   
   # Approve a proposal (replace 0 with actual proposal ID)
   dfx canister call demo approve '(0)'
   
   # List open proposals
   dfx canister call demo list_proposals
   
   # Check current config
   dfx canister call demo get_config
   ```

## Architecture

### Core Components

1. **Multisig<T>**: Generic multisig state machine
2. **Proposal<T>**: Individual proposal with voting state  
3. **MultisigManager<T, S>**: Optional wrapper with automatic persistence
4. **MultisigStorage<T>**: Trait for custom storage backends
5. **Owner management**: Principal-based access control
6. **Threshold voting**: Configurable approval requirements

### Storage Options

The library supports multiple storage patterns:

- **Pure in-memory**: Use `Multisig<T>` directly (no persistence)
- **No-op storage**: Use `MultisigManager<T, NoStorage>` (same as in-memory)
- **Custom storage**: Implement `MultisigStorage<T>` for your persistence needs
- **Stable structures**: Use provided examples for IC stable storage

### Workflow

1. Owner proposes an action with payload
2. Other owners vote to approve
3. When threshold is reached, action executes automatically
4. Proposal is marked as executed to prevent re-execution

## Crates

### ic-multisig-voting

Core library providing the multisig voting primitives. See [crates/ic-multisig-voting/README.md](crates/ic-multisig-voting/README.md) for details.

### ic-multisig-demo  

Demo canister showcasing configuration management via multisig. See [crates/ic-multisig-demo/README.md](crates/ic-multisig-demo/README.md) for details.

## Security Considerations

- **Owner verification**: All operations verify caller is an owner
- **Re-entrancy protection**: Proposals marked executed before payload execution
- **Immutable execution**: Executed proposals cannot be re-executed
- **Threshold enforcement**: Strict approval count validation

## Development

### Usage Patterns

#### Simple In-Memory Usage
```rust
use ic_multisig_voting::Multisig;

let mut multisig = Multisig::new(owners, threshold);
let proposal_id = multisig.propose(caller, payload)?;
if let Some(action) = multisig.approve(voter, proposal_id)? {
    // Execute the action
}
```

#### With Storage Manager
```rust
use ic_multisig_voting::MultisigManager;

// In-memory (no persistence)
let mut manager = MultisigManager::in_memory(owners, threshold);

// With custom storage
let storage = MyCustomStorage::new();
let mut manager = MultisigManager::with_storage(owners, threshold, storage)?;
```

#### Custom Storage Implementation
```rust
use ic_multisig_voting::MultisigStorage;

struct MyStorage;

impl<T> MultisigStorage<T> for MyStorage 
where T: CandidType + for<'de> Deserialize<'de>
{
    type Error = String;
    
    fn save(&mut self, multisig: &Multisig<T>) -> Result<(), Self::Error> {
        // Your persistence logic
    }
    
    fn load(&mut self) -> Result<Option<Multisig<T>>, Self::Error> {
        // Your loading logic
    }
}
```

### Running Tests

```bash
cargo test
```

### Adding New Payload Types

1. Define your payload enum/struct with Candid derive macros
2. Implement proposal and approval functions for your specific actions
3. Add execution logic in the approval handler

### Example Custom Payload

```rust
#[derive(CandidType, Deserialize, Clone)]
pub enum MyPayload {
    TransferTokens { to: Principal, amount: u64 },
    UpdateSettings { new_setting: String },
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.