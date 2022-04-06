use crate::trans::Chargeback;
use crate::trans::Deposit;
use crate::trans::Dispute;
use crate::trans::Resolve;
use crate::trans::Transfer;
use crate::trans::Withdrawal;

pub struct Account {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub locked: bool,
}

fn tx(t: &Transfer) -> u32 {
    match t {
        Transfer::Deposit(deposit) => deposit.tx,
        Transfer::Withdrawal(withdrawal) => withdrawal.tx,
    }
}

fn tx_type(t: &Transfer) -> String {
    match t {
        Transfer::Deposit(_) => "Deposit".to_string(),
        Transfer::Withdrawal(_) => "Withdrawal".to_string(),
    }
}

fn disputed_amount(t: &Transfer) -> f32 {
    match t {
        Transfer::Deposit(deposit) => deposit.amount,
        Transfer::Withdrawal(withdrawal) => withdrawal.amount,
    }
}

impl Account {
    pub fn total(&self) -> f32 {
        self.available + self.held
    }

    pub fn deposit(&mut self, d: &Deposit) -> Result<String, String> {
        if self.locked {
            return Err(format!("Account {} Locked", self.id));
        }
        self.available += d.amount;
        Ok(format!("Deposit: {} to account {}", d.amount, self.id))
    }

    pub fn withdrawal(&mut self, w: &Withdrawal) -> Result<String, String> {
        if self.locked {
            return Err(format!("Account {} Locked", self.id));
        }
        if self.available > w.amount {
            self.available -= w.amount;
            Ok(format!("Withdrawal: {} from account {}", w.amount, self.id))
        } else {
            Err(format!(
                "Withdrawal: Insufficient Funds to Withdraw {} from account {}",
                w.amount, self.id
            ))
        }
    }

    pub fn dispute(&mut self, d: &Dispute) -> Result<String, String> {
        if self.locked {
            return Err(format!("Account {} Locked", self.id));
        }
        let disputed_amount = disputed_amount(&d.tx);
        self.held += disputed_amount;
        self.available -= disputed_amount;
        Ok(format!("Dispute: {} {}", tx_type(&d.tx), tx(&d.tx)))
    }

    pub fn resolve(&mut self, r: &Resolve) -> Result<String, String> {
        if self.locked {
            return Err(format!("Account {} Locked", self.id));
        }
        let disputed_amount = disputed_amount(&r.tx.tx);
        if self.held >= disputed_amount {
            self.held -= disputed_amount;
            self.available += disputed_amount;
            Ok(format!(
                "Resolve: Dispute {} {}",
                tx_type(&r.tx.tx),
                tx(&r.tx.tx)
            ))
        } else {
            Err(format!(
                "Resolve: Dispute {} {} Failed as There are not enough funds held",
                tx_type(&r.tx.tx),
                tx(&r.tx.tx)
            ))
        }
    }

    pub fn chargeback(&mut self, c: &Chargeback) -> Result<String, String> {
        let disputed_amount = disputed_amount(&c.tx.tx);
        if self.held >= disputed_amount {
            self.available += if let Transfer::Withdrawal(_) = &c.tx.tx {
                self.held + disputed_amount
            } else {
                0.0
            };
            self.held -= disputed_amount;
            self.locked = true;
            Ok(format!(
                "Chargeback: Dispute {} {}",
                tx_type(&c.tx.tx),
                tx(&c.tx.tx)
            ))
        } else {
            Err(format!(
                "Chargeback: Dispute {} {} Failed as There are not enough funds held",
                tx_type(&c.tx.tx),
                tx(&c.tx.tx)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trans::Chargeback;
    use crate::trans::Deposit;
    use crate::trans::Dispute;
    use crate::trans::Resolve;
    use crate::trans::Transfer;
    use crate::trans::Withdrawal;

    #[test]
    fn test_account_locked() {
        let mut acc1 = Account {
            id: 1,
            available: 0.0,
            held: 0.0,
            locked: true,
        };

        let deposit = Deposit {
            tx: 1,
            amount: 100.0,
        };

        let result = acc1.deposit(&deposit);

        match result {
            Err(msg) => assert_eq!(msg, "Account 1 Locked"),
            Ok(_) => assert!(false),
        }

        assert_eq!(acc1.available, 0.0);
        assert_eq!(acc1.held, 0.0);
        assert_eq!(acc1.total(), 0.0);
        assert_eq!(acc1.locked, true);
    }

    #[test]
    fn test_deposit() {
        let mut acc1 = Account {
            id: 1,
            available: 0.0,
            held: 0.0,
            locked: false,
        };

        let deposit = Deposit {
            tx: 1,
            amount: 100.0,
        };

        let result = acc1.deposit(&deposit);

        match result {
            Ok(msg) => assert_eq!(msg, "Deposit: 100 to account 1"),
            Err(_) => assert!(false),
        }

        assert_eq!(acc1.available, 100.0);
        assert_eq!(acc1.held, 0.0);
        assert_eq!(acc1.total(), 100.0);
        assert_eq!(acc1.locked, false);
    }

    #[test]
    fn test_withdrawal() {
        let mut acc1 = Account {
            id: 1,
            available: 100.0,
            held: 0.0,
            locked: false,
        };

        let withdrawal = Withdrawal {
            tx: 1,
            amount: 50.0,
        };

        let result = acc1.withdrawal(&withdrawal);

        match result {
            Ok(msg) => assert_eq!(msg, "Withdrawal: 50 from account 1"),
            Err(_) => assert!(false),
        }

        assert_eq!(acc1.available, 50.0);
        assert_eq!(acc1.held, 0.0);
        assert_eq!(acc1.total(), 50.0);
        assert_eq!(acc1.locked, false);
    }

    #[test]
    fn test_dispute() {
        let mut acc1 = Account {
            id: 1,
            available: 100.0,
            held: 0.0,
            locked: false,
        };

        let dispute = Dispute {
            tx: Transfer::Withdrawal(Withdrawal {
                tx: 1,
                amount: 50.0,
            }),
        };

        let result = acc1.dispute(&dispute);

        match result {
            Ok(msg) => assert_eq!(msg, "Dispute: Withdrawal 1"),
            Err(_) => assert!(false),
        }

        assert_eq!(acc1.available, 50.0);
        assert_eq!(acc1.held, 50.0);
        assert_eq!(acc1.total(), 100.0);
        assert_eq!(acc1.locked, false);
    }

    #[test]
    fn test_resolve() {
        let mut acc1 = Account {
            id: 1,
            available: 100.0,
            held: 0.0,
            locked: false,
        };

        let dispute = Dispute {
            tx: Transfer::Withdrawal(Withdrawal {
                tx: 1,
                amount: 50.0,
            }),
        };

        let _ = acc1.dispute(&dispute);

        let resolve = Resolve { tx: dispute };

        let result = acc1.resolve(&resolve);

        match result {
            Ok(msg) => assert_eq!(msg, "Resolve: Dispute Withdrawal 1"),
            Err(_) => assert!(false),
        }

        assert_eq!(acc1.available, 100.0);
        assert_eq!(acc1.held, 0.0);
        assert_eq!(acc1.total(), 100.0);
        assert_eq!(acc1.locked, false);
    }

    #[test]
    fn test_resolve_no_enough_held() {
        let mut acc1 = Account {
            id: 1,
            available: 100.0,
            held: 0.0,
            locked: false,
        };

        let dispute = Dispute {
            tx: Transfer::Withdrawal(Withdrawal {
                tx: 1,
                amount: 50.0,
            }),
        };

        let resolve = Resolve { tx: dispute };

        let result = acc1.resolve(&resolve);

        match result {
            Err(msg) => assert_eq!(
                msg,
                "Resolve: Dispute Withdrawal 1 Failed as There are not enough funds held"
            ),
            Ok(_) => assert!(false),
        }

        assert_eq!(acc1.available, 100.0);
        assert_eq!(acc1.held, 0.0);
        assert_eq!(acc1.total(), 100.0);
        assert_eq!(acc1.locked, false);
    }

    #[test]
    fn test_chargeback() {
        let mut acc1 = Account {
            id: 1,
            available: 100.0,
            held: 0.0,
            locked: false,
        };

        let dispute = Dispute {
            tx: Transfer::Withdrawal(Withdrawal {
                tx: 1,
                amount: 50.0,
            }),
        };

        let _ = acc1.dispute(&dispute);

        let chargeback = Chargeback { tx: dispute };

        let result = acc1.chargeback(&chargeback);

        match result {
            Ok(msg) => assert_eq!(msg, "Chargeback: Dispute Withdrawal 1"),
            Err(_) => assert!(false),
        }

        assert_eq!(acc1.available, 150.0);
        assert_eq!(acc1.held, 0.0);
        assert_eq!(acc1.total(), 150.0);
        assert_eq!(acc1.locked, true);
    }

    #[test]
    fn test_chargeback_no_enough_held() {
        let mut acc1 = Account {
            id: 1,
            available: 100.0,
            held: 0.0,
            locked: false,
        };

        let dispute = Dispute {
            tx: Transfer::Withdrawal(Withdrawal {
                tx: 1,
                amount: 50.0,
            }),
        };

        let chargeback = Chargeback { tx: dispute };

        let result = acc1.chargeback(&chargeback);

        match result {
            Err(msg) => assert_eq!(
                msg,
                "Chargeback: Dispute Withdrawal 1 Failed as There are not enough funds held"
            ),
            Ok(_) => assert!(false),
        }

        assert_eq!(acc1.available, 100.0);
        assert_eq!(acc1.held, 0.0);
        assert_eq!(acc1.total(), 100.0);
        assert_eq!(acc1.locked, false);
    }
}
