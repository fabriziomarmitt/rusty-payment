extern crate csv;

mod acc;
mod trans;

use crate::acc::Account;
use crate::trans::settle;
use crate::trans::Transaction;

use std::collections::HashMap;
use std::env;

fn main() {
    let csv_file = env::args().nth(1).unwrap();
    let mut rdr = csv::Reader::from_path(csv_file).unwrap();
    let mut accounts = HashMap::new();
    let mut transactions = HashMap::<u32, Transaction>::new();
    for result in rdr.records() {
        let record = result.unwrap();

        let ttype = &record[0];
        let client: u16 = record[1].to_string().parse().unwrap();
        let tx: u32 = record[2].to_string().parse().unwrap();
        let amount: f32 = record[3].to_string().parse().unwrap_or_default();

        let acc: &mut Account = accounts.entry(client).or_insert(Account {
            id: client,
            available: 0.0,
            held: 0.0,
            locked: false,
        });
        let _transaction = settle(&ttype, acc, tx, amount, &mut transactions);
    }

    println!("client,available,held,total,locked");

    let mut sorted: Vec<_> = accounts.iter().collect();
    sorted.sort_by_key(|a| a.0);

    for (_, account) in sorted.iter() {
        println!(
            "{},{:.1},{:.1},{:.1},{}",
            account.id,
            account.available,
            account.held,
            account.total(),
            account.locked
        );
    }
}

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;

    #[test]
    fn test_transactions() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("rs").unwrap();

        cmd.arg("transactions.csv");
        cmd.assert().success().stdout(predicate::str::contains(
            "client,available,held,total,locked\n1,1.5,0.0,1.5,false\n2,2.0,0.0,2.0,false\n",
        ));

        Ok(())
    }

    #[test]
    fn test_chargeback() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("rs").unwrap();

        cmd.arg("chargeback.csv");
        cmd.assert().success().stdout(predicate::str::contains(
            "client,available,held,total,locked\n1,3.0,0.0,3.0,true\n2,2.0,0.0,2.0,false\n",
        ));

        Ok(())
    }

    #[test]
    fn test_resolve() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("rs").unwrap();

        cmd.arg("resolve.csv");
        cmd.assert().success().stdout(predicate::str::contains(
            "client,available,held,total,locked\n1,1.5,0.0,1.5,false\n2,2.0,0.0,2.0,false\n",
        ));

        Ok(())
    }
}
