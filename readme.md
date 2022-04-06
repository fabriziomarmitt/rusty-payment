# Rusty Payments

Requirements:
- Read CSV of transactions
- Update client accounts
- Handles Disputes and chargebacks
- Output accounts state as CSV
- transaction IDs are valid u32 values.
- Avoid scenarion: Mailicious make fiat funds, purchase and withdrawal BTC, and reverse deposit;

## Out-of-scope, Limitations and Assumptions:
- Persist Account state between program executions; once program executed, all memory is cleared, only output to stdout is displayed, but previous output from a previous execution will not be reused as a way to persist state, as handle race conditions for storing data in fs, is considered out-of-scope in the interest of time;
- Handle malformated csv inputs;
- Accounts with negative availability or negative held, and overdraft;
- Unlocking accounts; Once locked, that you cannot operate anymore on that account;
- You can only have one dispute per `Transfer` transaction; Currently, you can have multiple disputes, but each dispute overwrites the other, for same transaction; This is problably a security issue, and an open point for fraud. The reson for this limitation, is lack of understanding of the best business logic: Can a transaction have multiple disputes? At the same time or in series only? Not handle those cases was a decision in interest of time saving.
- tx is correct and will never collide.

## Open questions:

### Is it building?

Yes, the application builds with `cargo build`;

### Does it read and write data in in correct format?

Yes, there are tests in place in the `main.rs` that make sure formats are the same as expected.

### Does it handle all cases? Disputes, resolutions and chargebacks?

Yes, `acc.rs` implements `Account` struct that implements all operations over an account. However, `Account` itself does not handle the state of transactions, for example, `Account` can make a chargeback without a `Dispute`. 

To overcome that limitation, `trans.ts` implements the struct `Transaction` that must contain a `Transfer` (that can be `Withdrawal` or `Deposit`), and optionally, one `Dispute` and one `Settlement` (`Settlement` as the resolution of a `Dispute`, that can be `Resolve` or `Chargeback`). 

In other words, a `Transaction` can have one `Dispute` and one `Settlement`. Multiple disputes and multiples settlements will override previous occurances and mess up the state of accounts; This is a know limitation and listed in the `Limitations` section.

### For the cases you are handling are you handling them correctly? How do you know this? Did you test against sample data? If so, include it in the repo. Did you write unit tests for the complicated bits? Or are you using the type system to ensure correctness? 

In `acc.rs` the most common use cases scenarios are convered with tests, witch make me confident that cases are being handled, however I haven't stressed 100% test converage and corner cases might occur. I've create end2end tests in `main.rs` where I read sample CSV files `chargeback.csv`,`transactions.csv`, and `resolve.csv` to test all cases, but not stressing all possible scenarios.

To ensure correctness I've implemented a type system that make sure you cannot do a `Settlement` without a `Dispute`, also, cannot start a `Dispute` without a `Transfer`. 

### Are you doing something dangerous? Tell us why you chose to do it this way. How are you handling errors?

I think the most dangerous thing is the `Dispute`/`Settlement` model, where I allow overriding disputes and settlements. This is a door for fraud. Reason why I haven't invested much time on it is that I don't know what are the correct behaviour for those scenarios, like, should we allow multiple disputes for same transaction? Should them be parallel or serial? Can I chargeback a resolve or resolve a chargeback? I think spend more time I would need to make a serious amount of assumptions, that would only consume me time, but would not ensure correctnes of the program.

### Can you stream values through memory as opposed to loading the entire data set upfront? What if your code was bundled in a server, and these CSVs came from thousands of concurrent TCP streams?

Using csv reader I am ensureing that each line is being read at a time, without loading all the CSV file into memory, but only what is required to add a line into memory, that is flushed after being out of scope. 

This implements some memory efficiency, if a TCP server is receving CSV files as streams, currently, this program does not support it. The TCP server would need to wait all the CSV file to be received, same as a file, and then invoke this program with the CSV file as parameter, that then, would read the file, line by line, until end.

It is possible, however, to invoke the `settle` function from `trans.ts` in the context of a TCP server, where I could spawn a TCP Server that would wait until a line of a CSV file is received, then invoke `settle` and wait to next line. This would work and the limits of this program would be the amount of memory of the server to store accounts and transactions, but would be very performant to a certain extend.

### Maintainability

I think that clean code is one way to have readable code, and indeed a programmer spends more time reading code than writing code, that is why writing readable code is important. It is also a trade-off, the more readable is the code, the more time you need to spend to think in how to make that code more readable.

In that sense, I tried to use a more pragmatic approach, trying to make as readable as possible and as fast as possible, as it is a toy code. I tried to use the elements of readability:
- Small files, respecting single responsability for each file and the things inside of it;
- Small functions, allowing reader see the all function without scrolling;
- Short but meaniful names within it's context. Naming is hard, but I tried to add best short name in the right context. Used single letters variable names for very short lived variables, where is not needed to scroll to see all usages of that variable;