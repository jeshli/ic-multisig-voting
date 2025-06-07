# ic-multisig-voting

A generic multisignature voting library for Internet Computer canisters.

## Overview

This library provides a flexible multisig voting mechanism where multiple owners can propose and vote on arbitrary actions. Once a proposal receives enough approvals (threshold), it automatically executes.

## Features

- **Generic payload support**: Vote on any Candid-serializable data
- **Configurable thresholds**: Set minimum approvals required
- **Automatic execution**: Proposals execute when threshold is met
- **Re-entrancy protection**: Prevents double execution
- **Principal-based ownership**: Uses IC Principal for identity

## Usage

### Basic Setup

```rust
use ic_multisig_voting::{Multisig, Proposal};
use ic_cdk::export::Principal;

// Create a multisig with 3 owners requiring 2 approvals
let owners = vec![
    Principal::from_text("owner1").unwrap(),
    Principal::from_text("owner2").unwrap(), 
    Principal::from_text("owner3").unwrap(),
];
let mut multisig = Multisig::<MyPayload>::new(owners, 2);
```

### Proposing Actions

```rust
#[derive(CandidType, Deserialize, Clone)]
pub enum MyPayload {
    UpdateConfig(String),
    TransferFunds { to: Principal, amount: u64 },
}

// Owner proposes an action
let proposal_id = multisig.propose(
    caller_principal, 
    MyPayload::UpdateConfig("new_value".to_string())
);
```

### Voting on Proposals

```rust
// Other owners vote
if let Some(action) = multisig.approve(voter_principal, proposal_id) {
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

### Querying Proposals

```rust
// List all open (unexecuted) proposals
let open_proposals = multisig.list_open();
for proposal in open_proposals {
    println!("Proposal {}: {:?}", proposal.id, proposal.payload);
    println!("Approvals: {}/{}", proposal.approvals.len(), threshold);
}
```

## Integration with IC Canisters

### Thread-Local Storage Pattern

```rust
use std::cell::RefCell;

thread_local! {
    static MULTISIG: RefCell<Multisig<MyPayload>> = 
        RefCell::new(Multisig::new(vec![], 1));
}

fn with_multisig<F, R>(f: F) -> R 
where F: FnOnce(&mut Multisig<MyPayload>) -> R 
{
    MULTISIG.with(|ms| f(&mut ms.borrow_mut()))
}

#[ic_cdk::init]
fn init(owners: Vec<Principal>, threshold: u8) {
    MULTISIG.with(|ms| {
        *ms.borrow_mut() = Multisig::new(owners, threshold);
    });
}
```

### Canister Update Methods

```rust
#[ic_cdk::update]
fn propose_action(payload: MyPayload) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| ms.propose(caller, payload))
}

#[ic_cdk::update] 
fn approve_proposal(id: u64) {
    let caller = ic_cdk::caller();
    if let Some(action) = with_multisig(|ms| ms.approve(caller, id)) {
        execute_action(action);
    }
}

#[ic_cdk::query]
fn list_proposals() -> Vec<Proposal<MyPayload>> {
    with_multisig(|ms| ms.list_open().into_iter().cloned().collect())
}
```

## API Reference

### `Multisig<T>`

The main multisig state container.

#### Methods

- `new(owners: Vec<Principal>, threshold: u8) -> Self`
  - Creates new multisig with given owners and approval threshold
  - Panics if threshold is 0 or exceeds owner count

- `propose(caller: Principal, payload: T) -> ProposalId`  
  - Creates new proposal with given payload
  - Caller must be an owner
  - Proposer automatically approves their own proposal
  - Returns unique proposal ID

- `approve(caller: Principal, id: ProposalId) -> Option<T>`
  - Adds caller's approval to proposal
  - Returns `Some(payload)` if threshold reached and not previously executed
  - Returns `None` if more approvals needed or already executed
  - Caller must be an owner

- `list_open() -> Vec<&Proposal<T>>`
  - Returns all unexecuted proposals
  - Useful for querying current voting state

### `Proposal<T>`

Individual proposal data structure.

#### Fields

- `id: ProposalId` - Unique identifier
- `payload: T` - The proposed action
- `approvals: BTreeSet<Principal>` - Set of approving owners  
- `executed: bool` - Whether proposal has been executed

## Error Handling

The library uses `ic_cdk::trap()` for error conditions:

- **"caller is not an owner"**: Non-owner tried to propose/approve
- **"no such proposal"**: Attempted to approve non-existent proposal ID

## Security Features

### Re-entrancy Protection
Proposals are marked as executed before payload is returned, preventing double execution even if the execution logic calls back into the multisig.

### Owner Verification
All operations verify the caller is in the owner set before proceeding.

### Immutable Execution
Once executed, proposals cannot be executed again.

## Example Payload Types

### Configuration Management
```rust
#[derive(CandidType, Deserialize, Clone)]
pub struct Config {
    pub max_payload_size: u32,
    pub allowed_origins: Vec<String>,
}

#[derive(CandidType, Deserialize, Clone)]
pub enum ConfigPayload {
    UpdateConfig(Config),
    AddOwner(Principal),
    RemoveOwner(Principal),
}
```

### Asset Management
```rust
#[derive(CandidType, Deserialize, Clone)]
pub enum AssetPayload {
    Mint { to: Principal, amount: u64 },
    Burn { amount: u64 },
    Freeze { account: Principal },
}
```

## Dependencies

- `candid`: Candid type serialization
- `ic-cdk`: Internet Computer development kit
- Standard library collections (`BTreeMap`, `BTreeSet`)

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_workflow() {
        let owners = vec![Principal::anonymous()];
        let mut ms = Multisig::<u32>::new(owners, 1);
        
        let id = ms.propose(Principal::anonymous(), 42);
        let result = ms.approve(Principal::anonymous(), id);
        
        assert_eq!(result, Some(42));
    }
}
```