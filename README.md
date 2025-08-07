# ic-multisig-voting

A lightweight, generic multisignature voting library for Internet Computer canisters.

## Overview

This library provides a flexible multisig voting mechanism where multiple owners can propose and vote on arbitrary actions. Once a proposal reaches the required threshold of approvals, it automatically executes.

## Features

- **Generic payload support**: Vote on any Candid-serializable data
- **Configurable thresholds**: Set minimum approvals required  
- **Auto-execution**: Proposals execute when threshold is met
- **Re-entrancy protection**: Prevents double execution
- **Principal-based ownership**: Uses IC Principal for identity
- **Byte serialization**: Built-in support for persistent storage

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ic-multisig-voting = { path = "path/to/ic-multisig-voting" }
candid = "0.10"
ic-cdk = "0.16"
```

### Basic Usage

```rust
use ic_multisig_voting::{Multisig, Proposal};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum MyPayload {
    UpdateConfig(String),
    TransferFunds { to: Principal, amount: u64 },
}

// Create a multisig with 3 owners requiring 2 approvals
let owners = vec![
    Principal::from_text("owner1").unwrap(),
    Principal::from_text("owner2").unwrap(), 
    Principal::from_text("owner3").unwrap(),
];
let mut multisig = Multisig::<MyPayload>::new(owners, 2);

// Owner proposes an action
let proposal_id = multisig.propose(
    caller_principal, 
    MyPayload::UpdateConfig("new_value".to_string())
)?;

// Other owners vote
if let Some(action) = multisig.approve(voter_principal, proposal_id)? {
    // Threshold reached! Execute the action
    match action {
        MyPayload::UpdateConfig(value) => {
            // Apply configuration change
        },
        MyPayload::TransferFunds { to, amount } => {
            // Execute transfer
        },
    }
}
```

## IC Canister Integration

### Thread-Local Storage Pattern

```rust
use std::cell::RefCell;

thread_local! {
    static MULTISIG: RefCell<Option<Multisig<MyPayload>>> = RefCell::new(None);
}

fn with_multisig<F, R>(f: F) -> Result<R, String>
where F: FnOnce(&mut Multisig<MyPayload>) -> R 
{
    MULTISIG.with(|ms| {
        match ms.borrow_mut().as_mut() {
            Some(multisig) => Ok(f(multisig)),
            None => Err("Multisig not initialized".to_string()),
        }
    })
}

#[ic_cdk::init]
fn init(owners: Vec<Principal>, threshold: u8) {
    MULTISIG.with(|ms| {
        *ms.borrow_mut() = Some(Multisig::new(owners, threshold));
    });
}

#[ic_cdk::update]
fn propose_action(payload: MyPayload) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    with_multisig(|ms| ms.propose(caller, payload))
}

#[ic_cdk::update] 
fn approve_proposal(id: u64) -> Result<String, String> {
    let caller = ic_cdk::caller();
    let maybe_payload = with_multisig(|ms| ms.approve(caller, id))?;
    
    match maybe_payload {
        Some(action) => {
            execute_action(action);
            Ok(format!("Proposal {} executed", id))
        }
        None => Ok(format!("Proposal {} approved, waiting for more votes", id)),
    }
}

#[ic_cdk::query]
fn list_proposals() -> Result<Vec<Proposal<MyPayload>>, String> {
    with_multisig(|ms| Ok(ms.list_open().into_iter().cloned().collect()))
}
```

### Persistence with Stable Storage

```rust
use ic_stable_structures::storable::{Bound, Storable};

impl<T: CandidType + Clone + for<'de> Deserialize<'de>> Storable for Multisig<T> {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        match self.to_bytes() {
            Ok(bytes) => std::borrow::Cow::Owned(bytes),
            Err(_) => std::borrow::Cow::Borrowed(&[]), // Handle appropriately
        }
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self::from_bytes(bytes.as_ref()).unwrap() // Handle error appropriately
    }

    const BOUND: Bound = Bound::Unbounded;
}
```

## API Reference

### `Multisig<T>`

The main multisig state container.

#### Core Methods

- `new(owners: Vec<Principal>, threshold: u8) -> Self`
  - Creates new multisig with given owners and approval threshold
  - Panics if threshold is 0 or exceeds owner count

- `propose(caller: Principal, payload: T) -> Result<ProposalId, String>`  
  - Creates new proposal with given payload
  - Caller must be an owner
  - Proposer automatically approves their own proposal
  - Returns unique proposal ID

- `approve(caller: Principal, id: ProposalId) -> Result<Option<T>, String>`
  - Adds caller's approval to proposal
  - Returns `Some(payload)` if threshold reached and not previously executed
  - Returns `None` if more approvals needed or already executed
  - Caller must be an owner

#### Query Methods

- `list_open() -> Vec<&Proposal<T>>` - Returns all unexecuted proposals
- `get_proposal(id: ProposalId) -> Option<&Proposal<T>>` - Get proposal by ID
- `get_owners() -> &BTreeSet<Principal>` - Get current owners
- `get_threshold() -> u8` - Get current threshold

#### Owner Management

- `add_owner(owner: Principal) -> Result<(), String>` - Add new owner
- `remove_owner(owner: Principal) -> Result<(), String>` - Remove owner (if threshold allows)
- `set_threshold(new_threshold: u8) -> Result<(), String>` - Change approval threshold

#### Serialization

- `to_bytes(&self) -> Result<Vec<u8>, String>` - Serialize for storage
- `from_bytes(bytes: &[u8]) -> Result<Self, String>` - Deserialize from storage

### `Proposal<T>`

Individual proposal data structure.

```rust
pub struct Proposal<T> {
    pub id: ProposalId,
    pub payload: T,
    pub approvals: BTreeSet<Principal>,
    pub executed: bool,
}
```

## Error Handling

All methods return `Result` types with descriptive error messages:

- `"caller is not an owner"` - Non-owner tried to propose/approve
- `"no such proposal"` - Invalid proposal ID
- `"already an owner"` - Attempted to add existing owner
- `"removing owner would violate threshold"` - Can't remove owner if it would make threshold impossible
- `"invalid threshold"` - Threshold is 0 or exceeds owner count

## Security Features

- **Owner verification**: All operations verify caller is an owner
- **Re-entrancy protection**: Proposals marked executed before payload is returned
- **Immutable execution**: Executed proposals cannot be re-executed
- **Threshold enforcement**: Strict approval count validation

## Example Payload Types

### Configuration Management
```rust
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct Config {
    pub max_payload_size: u32,
    pub allowed_origins: Vec<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum ConfigPayload {
    UpdateConfig(Config),
    AddOwner(Principal),
    RemoveOwner(Principal),
    ChangeThreshold(u8),
}
```

### Asset Management
```rust
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum AssetPayload {
    Mint { to: Principal, amount: u64 },
    Burn { amount: u64 },
    Freeze { account: Principal },
    Unfreeze { account: Principal },
}
```

### Governance Actions
```rust
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub enum GovernancePayload {
    UpgradeCanister { wasm: Vec<u8> },
    SetController { controller: Principal },
    UpdateSettings { settings: CanisterSettings },
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_workflow() {
        let owners = vec![Principal::anonymous()];
        let mut ms = Multisig::<u32>::new(owners, 1);
        
        let id = ms.propose(Principal::anonymous(), 42).unwrap();
        let result = ms.approve(Principal::anonymous(), id).unwrap();
        
        assert_eq!(result, Some(42));
    }
    
    #[test]
    fn test_serialization_round_trip() {
        let owners = vec![Principal::anonymous()];
        let mut ms = Multisig::<String>::new(owners, 1);
        
        let _id = ms.propose(Principal::anonymous(), "test".to_string()).unwrap();
        
        let bytes = ms.to_bytes().unwrap();
        let restored = Multisig::<String>::from_bytes(&bytes).unwrap();
        
        assert_eq!(ms.get_threshold(), restored.get_threshold());
        assert_eq!(ms.get_owners(), restored.get_owners());
    }
}
```

Run tests with:
```bash
cargo test
```

## Dependencies

- `candid` - Candid type serialization
- `serde` - Serialization framework
- Standard library collections (`BTreeMap`, `BTreeSet`)

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Contributing

1. Fork the repository
2. Create a feature branch  
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request