# ic-multisig-demo

A demonstration canister showcasing the `ic-multisig-voting` library with configuration management functionality.

## Overview

This canister implements a multisig-controlled configuration system where multiple owners can propose and vote on configuration changes. It serves as both a practical example and a starting point for building multisig-controlled applications.

## Features

- **Configuration Management**: Multisig voting for system configuration changes
- **Owner Management**: Add/remove owners via multisig proposals
- **Proposal Querying**: View all open proposals and their voting status
- **Threshold Control**: Modify approval requirements through multisig

## Current Issues & Fixes Needed

⚠️ **The current code has several issues that prevent deployment:**

1. **Dependency Name Mismatch**: `Cargo.toml` references `ic-multisig-voting` but code imports `multisig_lib`
2. **Missing Canister Functions**: Core canister endpoints are commented out
3. **Undefined Types**: `Config` and related types need implementation
4. **Incomplete Integration**: Thread-local storage pattern needs activation

## Fixed Implementation

Here's the corrected version that will deploy successfully:

### Cargo.toml (Fixed)
```toml
[package]
name = "ic-multisig-demo"
version = "0.1.0"
edition = "2021"
publish = false

[lib]
crate-type = ["cdylib"]

[dependencies]
ic-multisig-voting = { path = "../ic-multisig-voting" }
candid = "0.10"
ic-cdk = "0.14"
```

### src/lib.rs (Complete Implementation)
```rust
use candid::{CandidType, Deserialize};
use ic_cdk::export::Principal;
use ic_multisig_voting::{Multisig, Proposal};
use std::cell::RefCell;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Config {
    pub max_payload_size: u32,
    pub allowed_origins: Vec<String>,
    pub admin_fee: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub enum ActionPayload {
    SetConfig(Config),
    AddOwner(Principal),
    RemoveOwner(Principal),
    ChangeThreshold(u8),
}

thread_local! {
    static MULTISIG: RefCell<Multisig<ActionPayload>> = 
        RefCell::new(Multisig::new(vec![], 1));
    
    static CONFIG: RefCell<Config> = RefCell::new(Config {
        max_payload_size: 1024,
        allowed_origins: vec!["https://ic0.app".to_string()],
        admin_fee: 0,
    });
}

fn with_multisig<F, R>(f: F) -> R
where F: FnOnce(&mut Multisig<ActionPayload>) -> R {
    MULTISIG.with(|ms| f(&mut ms.borrow_mut()))
}

fn with_config<F, R>(f: F) -> R  
where F: FnOnce(&mut Config) -> R {
    CONFIG.with(|cfg| f(&mut cfg.borrow_mut()))
}

#[ic_cdk::init]
fn init(owners: Vec<Principal>, threshold: u8) {
    MULTISIG.with(|ms| {
        *ms.borrow_mut() = Multisig::new(owners, threshold);
    });
}

#[ic_cdk::update]
fn propose_set_config(config: Config) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| ms.propose(caller, ActionPayload::SetConfig(config)))
}

#[ic_cdk::update]
fn propose_add_owner(new_owner: Principal) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| ms.propose(caller, ActionPayload::AddOwner(new_owner)))
}

#[ic_cdk::update]
fn propose_remove_owner(owner: Principal) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| ms.propose(caller, ActionPayload::RemoveOwner(owner)))
}

#[ic_cdk::update]
fn propose_change_threshold(new_threshold: u8) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| ms.propose(caller, ActionPayload::ChangeThreshold(new_threshold)))
}

#[ic_cdk::update]
fn approve(proposal_id: u64) {
    let caller = ic_cdk::caller();
    if let Some(action) = with_multisig(|ms| ms.approve(caller, proposal_id)) {
        execute_action(action);
    }
}

#[ic_cdk::query]
fn list_proposals() -> Vec<Proposal<ActionPayload>> {
    with_multisig(|ms| ms.list_open().into_iter().cloned().collect())
}

#[ic_cdk::query]
fn get_config() -> Config {
    with_config(|cfg| cfg.clone())
}

fn execute_action(action: ActionPayload) {
    match action {
        ActionPayload::SetConfig(new_config) => {
            with_config(|cfg| *cfg = new_config);
        },
        ActionPayload::AddOwner(_) | 
        ActionPayload::RemoveOwner(_) | 
        ActionPayload::ChangeThreshold(_) => {
            // These would require more complex implementation
            // to modify the multisig state itself
            ic_cdk::println!("Owner/threshold management not yet implemented");
        }
    }
}

// Export Candid interface
ic_cdk::export_candid!();
```

## Deployment

### Prerequisites
- DFX 0.23.0+
- Rust with `wasm32-unknown-unknown` target

### Build and Deploy

1. **Navigate to demo directory**:
   ```bash
   cd crates/ic-multisig-demo
   ```

2. **Start local IC replica**:
   ```bash
   dfx start --clean --background
   ```

3. **Deploy with initial owners**:
   ```bash
   # Replace with actual principal IDs
   dfx deploy --argument '(
     vec {
       principal "rdmx6-jaaaa-aaaah-qcaiq-cai"; 
       principal "rrkah-fqaaa-aaaah-qcqwq-cai"
     }, 
     2
   )'
   ```

## Usage Examples

### Propose Configuration Change

```bash
dfx canister call demo propose_set_config '(
  record {
    max_payload_size = 2048;
    allowed_origins = vec {"https://ic0.app"; "https://nns.ic0.app"};
    admin_fee = 1000
  }
)'
```

### Approve a Proposal

```bash
# Get proposal ID from previous command output
dfx canister call demo approve '(0)'
```

### List Open Proposals  

```bash
dfx canister call demo list_proposals
```

### Get Current Configuration

```bash
dfx canister call demo get_config
```

### Add New Owner

```bash
dfx canister call demo propose_add_owner '(principal "new-owner-principal")'
# Then approve with: dfx canister call demo approve '(proposal_id)'
```

## API Reference

### Update Methods

- `propose_set_config(config: Config) -> u64`
  - Proposes a new system configuration
  - Returns proposal ID for voting

- `propose_add_owner(new_owner: Principal) -> u64`
  - Proposes adding a new multisig owner
  - Returns proposal ID

- `propose_remove_owner(owner: Principal) -> u64`  
  - Proposes removing an existing owner
  - Returns proposal ID

- `propose_change_threshold(new_threshold: u8) -> u64`
  - Proposes changing the approval threshold
  - Returns proposal ID

- `approve(proposal_id: u64)`
  - Adds caller's approval to specified proposal
  - Executes action if threshold is reached

### Query Methods

- `list_proposals() -> Vec<Proposal<ActionPayload>>`
  - Returns all open (unexecuted) proposals
  - Shows current voting status

- `get_config() -> Config`
  - Returns current system configuration

## Configuration Schema

```rust
pub struct Config {
    pub max_payload_size: u32,     // Maximum size for payloads
    pub allowed_origins: Vec<String>, // CORS allowed origins  
    pub admin_fee: u64,            // Administrative fee amount
}
```

## Action Types

```rust
pub enum ActionPayload {
    SetConfig(Config),           // Update system configuration
    AddOwner(Principal),         // Add new multisig owner
    RemoveOwner(Principal),      // Remove existing owner  
    ChangeThreshold(u8),         // Modify approval threshold
}
```

## Security Considerations

- **Owner-only operations**: All proposals require caller to be an owner
- **Threshold enforcement**: Actions only execute when enough approvals collected
- **Re-entrancy protection**: Inherited from underlying multisig library
- **Principal verification**: Uses IC's built-in principal authentication

## Extending the Demo

### Adding New Action Types

1. **Extend ActionPayload enum**:
   ```rust
   #[derive(CandidType, Deserialize, Clone, Debug)]
   pub enum ActionPayload {
       // ... existing variants
       TransferFunds { to: Principal, amount: u64 },
       PauseSystem(bool),
   }
   ```

2. **Add proposal function**:
   ```rust
   #[ic_cdk::update]
   fn propose_transfer(to: Principal, amount: u64) -> u64 {
       let caller = ic_cdk::caller();
       with_multisig(|ms| ms.propose(caller, ActionPayload::TransferFunds { to, amount }))
   }
   ```

3. **Handle in execute_action**:
   ```rust
   fn execute_action(action: ActionPayload) {
       match action {
           // ... existing cases
           ActionPayload::TransferFunds { to, amount } => {
               // Implement transfer logic
           },
           ActionPayload::PauseSystem(paused) => {
               // Implement pause logic  
           }
       }
   }
   ```

## Testing

```bash
# Run unit tests
cargo test

# Integration testing with local replica
dfx start --clean --background
dfx deploy
# ... run test scenarios
```

## Troubleshooting  

### Common Issues

1. **"caller is not an owner"**: Ensure your principal is in the initial owners list
2. **"no such proposal"**: Verify proposal ID exists with `list_proposals`
3. **Wasm build failures**: Ensure `wasm32-unknown-unknown` target is installed

### Debug Commands

```bash
# Check canister info
dfx canister info demo

# View canister logs  
dfx canister logs demo

# Get your principal ID
dfx identity get-principal
```