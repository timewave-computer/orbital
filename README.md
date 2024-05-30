# Orbital

```sh
 _______  ______    _______  ___   _______  _______  ___     
|       ||    _ |  |  _    ||   | |       ||   _   ||   |    
|   _   ||   | ||  | |_|   ||   | |_     _||  |_|  ||   |    
|  | |  ||   |_||_ |       ||   |   |   |  |       ||   |    
|  |_|  ||    __  ||  _   | |   |   |   |  |       ||   |___ 
|       ||   |  | || |_|   ||   |   |   |  |   _   ||       |
|_______||___|  |_||_______||___|   |___|  |__| |__||_______|
```

## Design

Orbital is an intent based cross chain auction system.

It involves two key concepts - account and auction.

Once deposit funds into their system account, they can submit intent-based orders.

An example intent order may look as follows:

```json
{
    ask_domain: "neutron",
    ask_coin: coin(10, "untrn"),
    offer_domain: "juno",
    offer_coin: coin(100, "ujuno"),
}
```

From users perspective, orbital offers the following functionality:

- deposit funds (banksend into desired proxy address)
- submit intent
- withdraw funds

Orbital enables marketmakers to fulfil user intents and capture the arbitrage:

- submit a slashable bond
- deposit funds

### Account

Users who wish to use the system must first create their system account.

Simply creating an account will create a neutron account, which is the instance of the contract.

In order to take part in cross-chain orders, users must register new domains.
This will instruct the system account to initiate a proxy account on a remote chain.

For now, this is done using polytone.

After polytone proxy is created and account receives the callback, users can bank-send funds
into the created proxy address.

This enables users to interact with a single contract address on neutron in order to control
an account on any chain that supports cosmwasm (polytone).

The plan was to have an option to chose between an ICA and Polytone account type for each domain,
but we ran out of time.

- some chains support cosmwasm, but not ICA host, and vice versa
- having the option to chose between two types of remote chain accounts would enable to cover most domains

### Double accounting

In order to enable the best user experience, system uses an internal double accounting ledger.

Once funds are deposited, a remote chain proxy balances sync is initiated.
This means account will send a balances query to its proxy on the specified domain.
Upon callback, internal ledger is updated to reflect the latest balances.

Internal user balances ledger gets updated in one of the following ways:

#### user deposits funds into the system

user deposits funds and remote chain balances are synced

#### user withdraws funds from the system

user withdraws funds from the system into a specified address

#### user submits an intent

user submits an intent, which is then fulfilled by a solver.

solver will deposit user ask funds into the destination address specified by the user,
on the destination domain.

account will then transfer the user offer funds to the solver specified address on the offer domain.

after that, the internal ledger is synced by querying both offer and ask domain proxy accounts.

### Auction

an English (ascending) auction is used for the system.

once user submits an intent to their account, account will perform an internal ledger validation.

if user balance on the offer domain is sufficient to fulfill their part of the intent, account will
instruct the auction to begin an auction cycle.

solvers can begin querying the auction and submitting their bids.

after auction period (sec/block based) is over, the winning solver bid enters the fulfilment period.

solvers are given a period of time during which they have to do the following:

1. deposit the winning bid amount into users destination address
1. submit a proof to the account to confirm the fulfilment

after account validates that the solver indeed delivered on the promise made when bidding,
account will submit a bank transfer message to the users offer domain proxy account. this
will transfer the offer funds to the marketmaker.

### Slashing

to prevent malicious solvers from submitting bogus bids that they do not intend (sigh) to fulfil,
solvers must bond an amount of tokens to the system.

bids are only accepted from solvers with an active bond.

if solver fails to deliver on their bid during the fulfilment period, auction will claim their bid.

in order to submit new bids, solvers must re-bond.
