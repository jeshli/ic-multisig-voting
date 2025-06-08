// lib.rs - Simplified multisig voting library with byte serialization
use candid::{CandidType, Decode, Encode, Principal};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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

impl<T: CandidType + Clone + for<'de> Deserialize<'de>> Multisig<T> {
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

    /// Serialize to bytes for storage
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Encode!(self).map_err(|e| format!("Failed to encode multisig: {}", e))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Decode!(bytes, Self).map_err(|e| format!("Failed to decode multisig: {}", e))
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

// Optional: Example of how users can implement Storable trait themselves
// This is just documentation - users should copy this to their own code
//
// use ic_stable_structures::storable::{Bound, Storable};
//
// impl<T: CandidType + Clone + for<'de> Deserialize<'de>> Storable for Multisig<T> {
//     fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
//         match self.to_bytes() {
//             Ok(bytes) => std::borrow::Cow::Owned(bytes),
//             Err(_) => std::borrow::Cow::Borrowed(&[]), // or panic!, depending on your preference
//         }
//     }
//
//     fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
//         Self::from_bytes(bytes.as_ref()).unwrap() // or handle error appropriately
//     }
//
//     const BOUND: Bound = Bound::Unbounded;
// }

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
    fn test_serialization() {
        let owners = vec![Principal::anonymous()];
        let mut ms = Multisig::<u32>::new(owners, 1);

        let id = ms.propose(Principal::anonymous(), 42).unwrap();

        // Serialize to bytes
        let bytes = ms.to_bytes().unwrap();

        // Deserialize from bytes
        let mut restored_ms = Multisig::<u32>::from_bytes(&bytes).unwrap();

        // Should be able to approve the proposal
        let result = restored_ms.approve(Principal::anonymous(), id).unwrap();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_round_trip_serialization() {
        // Create three different principals so we can test partial approval
        let owner1 = Principal::anonymous();
        let owner2 = Principal::from_slice(&[1, 2, 3, 4]);
        let owner3 = Principal::from_slice(&[5, 6, 7, 8]);
        let owners = vec![owner1, owner2, owner3];

        // Set threshold to 3 so proposals don't execute with just 2 approvals
        let mut ms = Multisig::<String>::new(owners.clone(), 3);

        // Add some proposals
        let id1 = ms.propose(owner1, "First proposal".to_string()).unwrap();
        let id2 = ms.propose(owner2, "Second proposal".to_string()).unwrap();

        // Partially approve id1 - should not execute yet (needs 3 approvals, has 2)
        let approve_result = ms.approve(owner2, id1).unwrap();
        assert_eq!(approve_result, None); // Should not execute yet

        // Check state before serialization
        let prop1_before = ms.get_proposal(id1).unwrap();
        assert_eq!(prop1_before.approvals.len(), 2); // proposer + one approval
        assert!(!prop1_before.executed);

        // Serialize
        let bytes = ms.to_bytes().unwrap();

        // Deserialize
        let restored_ms = Multisig::<String>::from_bytes(&bytes).unwrap();

        // Verify state
        assert_eq!(restored_ms.get_owners(), &owners.into_iter().collect());
        assert_eq!(restored_ms.get_threshold(), 3);
        assert_eq!(restored_ms.list_open().len(), 2); // Both proposals should be open

        let prop1_after = restored_ms.get_proposal(id1).unwrap();
        assert_eq!(prop1_after.approvals.len(), 2); // proposer + one approval
        assert!(!prop1_after.executed);

        let prop2_after = restored_ms.get_proposal(id2).unwrap();
        assert_eq!(prop2_after.approvals.len(), 1); // just proposer
        assert!(!prop2_after.executed);
    }

    #[test]
    fn test_serialization_with_executed_proposal() {
        let owner = Principal::anonymous();
        let mut ms = Multisig::<u32>::new(vec![owner], 1);

        // Create and execute a proposal
        let id = ms.propose(owner, 42).unwrap();
        let result = ms.approve(owner, id).unwrap();
        assert_eq!(result, Some(42));

        // Serialize
        let bytes = ms.to_bytes().unwrap();

        // Deserialize
        let restored_ms = Multisig::<u32>::from_bytes(&bytes).unwrap();

        // Verify executed proposal is preserved
        let prop = restored_ms.get_proposal(id).unwrap();
        assert!(prop.executed);
        assert_eq!(prop.payload, 42);
        assert_eq!(restored_ms.list_open().len(), 0); // No open proposals
    }
}