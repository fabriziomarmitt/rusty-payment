use crate::acc::Account;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Deposit {
    pub tx: u32,
    pub amount: f32,
}

#[derive(Clone)]
pub struct Withdrawal {
    pub tx: u32,
    pub amount: f32,
}

#[derive(Clone)]
pub enum Transfer {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
}

#[derive(Clone)]
pub struct Dispute {
    pub tx: Transfer,
}

#[derive(Clone)]
pub struct Resolve {
    pub tx: Dispute,
}

#[derive(Clone)]
pub struct Chargeback {
    pub tx: Dispute,
}

#[derive(Clone)]
pub enum Settlement {
    Resolve(Resolve),
    Chargeback(Chargeback),
}

#[derive(Clone)]
pub struct Transaction {
    pub transfer: Transfer,
    pub dispute: Option<Dispute>,
    pub settlement: Option<Settlement>,
}

pub fn settle(
    ttype: &str,
    acc: &mut Account,
    tx: u32,
    amount: f32,
    transactions: &mut HashMap<u32, Transaction>,
) -> Result<std::string::String, std::string::String> {
    match ttype {
        "deposit" => {
            let d = Deposit {
                tx: tx,
                amount: amount,
            };
            let rs = acc.deposit(&d);
            let t = Transaction {
                transfer: Transfer::Deposit(d),
                dispute: None,
                settlement: None,
            };
            transactions.insert(tx, t);
            rs
        }
        "withdrawal" => {
            let w = Withdrawal {
                tx: tx,
                amount: amount,
            };
            let rs = acc.withdrawal(&w);
            let t = Transaction {
                transfer: Transfer::Withdrawal(w),
                dispute: None,
                settlement: None,
            };
            transactions.insert(tx, t);
            rs
        }
        "dispute" => {
            let entry = transactions.entry(tx);
            match entry {
                Occupied(mut e) => {
                    let transaction = e.get_mut();
                    let d = Dispute {
                        tx: transaction.transfer.clone(),
                    };
                    let rs = acc.dispute(&d);
                    transaction.dispute = Some(d);
                    rs
                }
                Vacant(_) => Err(format!(
                    "Cannot find Transaction {} to Start a Dispute",
                    tx.to_string()
                )),
            }
        }
        "resolve" => {
            let entry = transactions.entry(tx);
            match entry {
                Occupied(mut e) => {
                    let transaction = e.get_mut();
                    match &transaction.dispute {
                        Some(dispute) => {
                            let d = Resolve {
                                tx: dispute.clone(),
                            };
                            let rs = acc.resolve(&d);
                            transaction.settlement = Some(Settlement::Resolve(d));
                            rs
                        }
                        None => Err(format!("Dispute for Transfer {} not found", tx.to_string())),
                    }
                }
                Vacant(_) => Err(format!(
                    "Cannot find Transaction {} to Resolve",
                    tx.to_string()
                )),
            }
        }
        "chargeback" => {
            let entry = transactions.entry(tx);
            match entry {
                Occupied(mut e) => {
                    let transaction = e.get_mut();
                    match &transaction.dispute {
                        Some(dispute) => {
                            let c = Chargeback {
                                tx: dispute.clone(),
                            };
                            let rs = acc.chargeback(&c);
                            transaction.settlement = Some(Settlement::Chargeback(c));
                            rs
                        }
                        None => Err(format!("Dispute for Transfer {} not found", tx.to_string())),
                    }
                }
                Vacant(_) => Err(format!(
                    "Cannot find Transaction {} to Resolve",
                    tx.to_string()
                )),
            }
        }
        _ => Err(format!("Unknown Transaction {}", ttype)),
    }
}
