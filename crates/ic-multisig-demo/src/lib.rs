// demo/lib.rs - Fixed version with proper error handling

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

/// Helper function to interact with multisig state
fn with_multisig<F, R>(f: F) -> R
where F: FnOnce(&mut Multisig<ActionPayload>) -> R {
    MULTISIG.with(|ms| f(&mut ms.borrow_mut()))
}

/// Helper function to interact with config state
fn with_config<F, R>(f: F) -> R
where F: FnOnce(&mut Config) -> R {
    CONFIG.with(|cfg| f(&mut cfg.borrow_mut()))
}

/// Initialize the canister with initial owners and threshold
#[ic_cdk::init]
fn init(owners: Vec<Principal>, threshold: u8) {
    MULTISIG.with(|ms| {
        *ms.borrow_mut() = Multisig::new(owners, threshold);
    });
}

/// Propose a new system configuration
#[ic_cdk::update]
fn propose_set_config(config: Config) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| {
        ms.propose(caller, ActionPayload::SetConfig(config))
            .unwrap_or_else(|e| ic_cdk::trap(&e))
    })
}

/// Propose adding a new owner
#[ic_cdk::update]
fn propose_add_owner(new_owner: Principal) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| {
        ms.propose(caller, ActionPayload::AddOwner(new_owner))
            .unwrap_or_else(|e| ic_cdk::trap(&e))
    })
}

/// Propose removing an existing owner
#[ic_cdk::update]
fn propose_remove_owner(owner: Principal) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| {
        ms.propose(caller, ActionPayload::RemoveOwner(owner))
            .unwrap_or_else(|e| ic_cdk::trap(&e))
    })
}

/// Propose changing the approval threshold
#[ic_cdk::update]
fn propose_change_threshold(new_threshold: u8) -> u64 {
    let caller = ic_cdk::caller();
    with_multisig(|ms| {
        ms.propose(caller, ActionPayload::ChangeThreshold(new_threshold))
            .unwrap_or_else(|e| ic_cdk::trap(&e))
    })
}

/// Approve a proposal by ID
#[ic_cdk::update]
fn approve(proposal_id: u64) {
    let caller = ic_cdk::caller();

    let result = with_multisig(|ms| ms.approve(caller, proposal_id));

    match result {
        Ok(Some(action)) => {
            execute_action(action);
        },
        Ok(None) => {
            // Successfully voted, but threshold not yet reached
            ic_cdk::println!("Vote recorded. Waiting for more approvals.");
        },
        Err(e) => {
            ic_cdk::trap(&e);
        }
    }
}

/// List all open (unexecuted) proposals
#[ic_cdk::query]
fn list_proposals() -> Vec<Proposal<ActionPayload>> {
    with_multisig(|ms| ms.list_open().into_iter().cloned().collect())
}

/// Get the current system configuration
#[ic_cdk::query]
fn get_config() -> Config {
    with_config(|cfg| cfg.clone())
}

/// Get multisig info (owners, threshold, etc.)
#[ic_cdk::query]
fn get_multisig_info() -> MultisigInfo {
    with_multisig(|ms| MultisigInfo {
        owners: ms.get_owners().iter().cloned().collect(),
        threshold: ms.get_threshold(),
        open_proposal_count: ms.list_open().len() as u64,
    })
}

#[derive(CandidType, Deserialize)]
pub struct MultisigInfo {
    pub owners: Vec<Principal>,
    pub threshold: u8,
    pub open_proposal_count: u64,
}

/// Execute an approved action
fn execute_action(action: ActionPayload) {
    match action {
        ActionPayload::SetConfig(new_config) => {
            with_config(|cfg| *cfg = new_config);
            ic_cdk::println!("Configuration updated successfully");
        },
        ActionPayload::AddOwner(new_owner) => {
            with_multisig(|ms| {
                if let Err(e) = ms.add_owner(new_owner) {
                    ic_cdk::println!("Failed to add owner: {}", e);
                } else {
                    ic_cdk::println!("Owner {} added successfully", new_owner);
                }
            });
        },
        ActionPayload::RemoveOwner(owner) => {
            with_multisig(|ms| {
                if let Err(e) = ms.remove_owner(owner) {
                    ic_cdk::println!("Failed to remove owner: {}", e);
                } else {
                    ic_cdk::println!("Owner {} removed successfully", owner);
                }
            });
        },
        ActionPayload::ChangeThreshold(new_threshold) => {
            with_multisig(|ms| {
                if let Err(e) = ms.set_threshold(new_threshold) {
                    ic_cdk::println!("Failed to change threshold: {}", e);
                } else {
                    ic_cdk::println!("Threshold changed to {} successfully", new_threshold);
                }
            });
        }
    }
}

// Export the Candid interface
ic_cdk::export_candid!();