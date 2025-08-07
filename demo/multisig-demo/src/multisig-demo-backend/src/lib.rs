// demo/lib.rs - Ultra simple demo
use candid::{CandidType, Principal};
use ic_cdk::{init, query, update};
use ic_multisig_voting::{Multisig, Proposal};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub enum DemoPayload {
    UpdateMessage(String),
}

thread_local! {
    static MULTISIG: RefCell<Option<Multisig<DemoPayload>>> = RefCell::new(None);
    static MESSAGE: RefCell<String> = RefCell::new("Hello World".to_string());
}

fn with_multisig<F, R>(f: F) -> Result<R, String>
where F: FnOnce(&mut Multisig<DemoPayload>) -> R {
    MULTISIG.with(|ms| {
        match ms.borrow_mut().as_mut() {
            Some(multisig) => Ok(f(multisig)),
            None => Err("Not initialized".to_string()),
        }
    })
}

#[init]
fn init(owners: Vec<Principal>, threshold: u8) {
    MULTISIG.with(|ms| {
        *ms.borrow_mut() = Some(Multisig::new(owners, threshold));
    });
}

#[update]
fn propose(message: String) -> Result<u64, String> {
    let caller = ic_cdk::caller();
    with_multisig(|ms| ms.propose(caller, DemoPayload::UpdateMessage(message)))
}

#[update]
fn approve(id: u64) -> Result<String, String> {
    let caller = ic_cdk::caller();

    let maybe_payload = with_multisig(|ms| ms.approve(caller, id))?;

    match maybe_payload {
        Some(DemoPayload::UpdateMessage(msg)) => {
            MESSAGE.with(|m| *m.borrow_mut() = msg.clone());
            Ok(format!("Proposal {} executed: message updated to '{}'", id, msg))
        }
        None => Ok(format!("Proposal {} approved, waiting for more votes", id)),
    }
}

#[query]
fn get_message() -> String {
    MESSAGE.with(|m| m.borrow().clone())
}

#[query]
fn list_proposals() -> Result<Vec<Proposal<DemoPayload>>, String> {
    with_multisig(|ms| Ok(ms.list_open().into_iter().cloned().collect()))
}

#[query]
fn whoami() -> Principal {
    ic_cdk::caller()
}


ic_cdk::export_candid!();