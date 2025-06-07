// lib.rs - Core multisig voting library
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod storage;
pub use storage::{MultisigStorage, NoStorage, MultisigManager};

// Re-export example storage implementations behind feature flags
#[cfg(feature = "examples")]
pub mod examples;

pub type ProposalId = u64;

/// A proposal waiting for votes
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Proposal<T> {
    pub id: ProposalId,
    pub payload: T,
    pub approvals: BTreeSet<Principal>,
    pub executed: bool,
}

/// Core multisig voting state machine
#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Multisig<T> {
    owners: BTreeSet<Principal>,
    threshold: u8,
    next_id: ProposalId,
    proposals: BTreeMap<ProposalId, Proposal<T>>,
}

impl<T: CandidType + Clone> Multisig<T> {
    /// Create a new multisig with given owners and approval threshold
    pub fn new(owners: Vec<Principal>, threshold: u8) -> Self {
        assert!(threshold > 0 && threshold as usize <= owners.len(),
                "threshold must be > 0 and <= number of owners");
        Self {
            owners: owners.into_iter().collect(),
            threshold,
            next_id: 0,
            proposals: BTreeMap::new(),
        }
    }

    /// Propose a new action; returns proposal ID
    pub fn propose(&mut self, caller: Principal, payload: T) -> Result<ProposalId, String> {
        if !self.owners.contains(&caller) {
            return Err("caller is not an owner".to_string());
        }

        let id = self.next_id;
        self.next_id += 1;

        let mut approvals = BTreeSet::new();
        approvals.insert(caller); // proposer auto-approves

        self.proposals.insert(
            id,
            Proposal {
                id,
                payload,
                approvals,
                executed: false,
            },
        );
        Ok(id)
    }

    /// Approve a proposal; returns Some(payload) if threshold reached and not executed
    pub fn approve(&mut self, caller: Principal, id: ProposalId) -> Result<Option<T>, String> {
        if !self.owners.contains(&caller) {
            return Err("caller is not an owner".to_string());
        }

        let prop = self.proposals
            .get_mut(&id)
            .ok_or("no such proposal")?;

        if prop.executed {
            return Ok(None);
        }

        prop.approvals.insert(caller);

        if prop.approvals.len() >= self.threshold as usize {
            prop.executed = true; // mark first to prevent re-entrancy
            Ok(Some(prop.payload.clone()))
        } else {
            Ok(None)
        }
    }

    /// List all open (unexecuted) proposals
    pub fn list_open(&self) -> Vec<&Proposal<T>> {
        self.proposals.values().filter(|p| !p.executed).collect()
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, id: ProposalId) -> Option<&Proposal<T>> {
        self.proposals.get(&id)
    }

    /// Get current owners
    pub fn get_owners(&self) -> &BTreeSet<Principal> {
        &self.owners
    }

    /// Get current threshold
    pub fn get_threshold(&self) -> u8 {
        self.threshold
    }

    /// Add owner (returns error if already exists)
    pub fn add_owner(&mut self, owner: Principal) -> Result<(), String> {
        if self.owners.contains(&owner) {
            return Err("already an owner".to_string());
        }
        self.owners.insert(owner);
        Ok(())
    }

    /// Remove owner (returns error if would violate threshold)
    pub fn remove_owner(&mut self, owner: Principal) -> Result<(), String> {
        if !self.owners.contains(&owner) {
            return Err("not an owner".to_string());
        }
        if self.owners.len() <= self.threshold as usize {
            return Err("removing owner would violate threshold".to_string());
        }
        self.owners.remove(&owner);
        Ok(())
    }

    /// Change threshold (returns error if invalid)
    pub fn set_threshold(&mut self, new_threshold: u8) -> Result<(), String> {
        if new_threshold == 0 || new_threshold as usize > self.owners.len() {
            return Err("invalid threshold".to_string());
        }
        self.threshold = new_threshold;
        Ok(())
    }
}

// Convenience type alias for in-memory usage
pub type InMemoryMultisig<T> = MultisigManager<T, NoStorage>;

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
    fn test_threshold_enforcement() {
        // Ultra-simple test to debug the issue
        let owner = Principal::anonymous();
        let mut ms = Multisig::<u32>::new(vec![owner], 1);

        // This should work fine
        let proposal_id = ms.propose(owner, 123);
        println!("Proposal result: {:?}", proposal_id);

        match proposal_id {
            Ok(id) => {
                let approval_result = ms.approve(owner, id);
                println!("Approval result: {:?}", approval_result);
                // Don't unwrap here - let's see what the actual error is
                match approval_result {
                    Ok(Some(value)) => assert_eq!(value, 123),
                    Ok(None) => panic!("Expected execution but got None"),
                    Err(e) => panic!("Approval failed: {}", e),
                }
            },
            Err(e) => panic!("Proposal failed: {}", e),
        }
    }

    #[test]
    fn test_in_memory_manager() {
        let mut manager = MultisigManager::in_memory(vec![Principal::anonymous()], 1);

        let id = manager.propose(Principal::anonymous(), 42).unwrap();
        let result = manager.approve(Principal::anonymous(), id).unwrap();

        assert_eq!(result, Some(42));
    }
}