use candid::{CandidType, Decode, Nat};
use ic_cdk::{api, trap};
use ic_cdk::{init, query, update};
use icrc_ledger_types::icrc21::{
    requests::{ConsentMessageRequest, ConsentMessageMetadata},
    responses::{ConsentInfo, ConsentMessage},
};
// use icrc_ledger_types::icrc28::TrustedOriginsResponse;
use serde::Deserialize;
use std::cell::RefCell;

thread_local! { static COUNTER: RefCell<u64> = RefCell::new(0); }


// ---- ICRC-28 placeholder ----
#[derive(candid::CandidType, serde::Deserialize)]
struct TrustedOriginsResponse {
    origins: Vec<String>,
}

// ---------- demo business logic ----------
#[update]
fn increment(by: u64) -> u64 {
    COUNTER.with(|c| {
        let mut v = c.borrow_mut();
        *v += by;
        *v
    })
}

#[query]
fn get() -> u64 {
    COUNTER.with(|c| *c.borrow())
}

// ---------- ICRC-21 dynamic consent ----------
#[derive(CandidType, Deserialize)]
struct IncArgs(u64);

#[query(name = "icrc21_call_consent_message")]
fn consent(req: ConsentMessageRequest) -> ConsentInfo {
    // (optional) decode `req.arg` if you want to show dynamic values
    let markdown = format!(
        "This action will **increment** the demo counter by the supplied amount.\n\n\
         _Target method_: `{}`",
        req.method
    );

    ConsentInfo {
        consent_message: ConsentMessage::GenericDisplayMessage(markdown),
        metadata: ConsentMessageMetadata {
            language: "en".into(),
            utc_offset_minutes: None,
        },
    }
}

// ---------- ICRC-28 trusted front-ends ----------
#[query(name = "icrc28_trusted_origins")]
fn trusted() -> TrustedOriginsResponse {
    TrustedOriginsResponse {
        origins: vec![
            // dev-server URL where your front-end runs
            "http://localhost:5173".into(),
        ],
    }
}

#[init]
fn init() {}



ic_cdk::export_candid!();