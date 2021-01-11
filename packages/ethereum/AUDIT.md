# Scope

To be audited:

- [HoprToken.sol](./contracts/HoprToken.sol)
- [ERC777Snapshot.sol](./contracts/ERC777/ERC777Snapshot.sol)
- [HoprDistributor.sol](./contracts/HoprDistributor.sol)

# The main smart contracts

## HoprToken

[[source code](./contracts/HoprToken.sol)]

HoprToken is an ERC777 compatible token, it also allows for quering balance data at a specific block.
To achieve that we have [extended](./contracts/ERC777/ERC777Snapshot.sol) OZ's ERC777 contract and implemented
a snapshotting mechanism similar to the [minime token](https://github.com/Giveth/minime/blob/ea04d950eea153a04c51fa510b068b9dded390cb/contracts/MiniMeToken.sol).

The HoprToken smart contract should:

- should be ERC20 compatible
- should be ERC777 compatible
- a user with role "MINTER_ROLE" should be able to mint tokens
- should return the balance of a user at specified block number
- should return the totalSupply at specified block number

## HoprDistributor

[[source code](./contracts/HoprDistributor.sol)]

HoprDistributor is a smart contract that allows us to distribute HOPR tokens.
It works by creating schedules and allocations, then a user who is eligible can
claim his/hers tokens.
Schedules are named and define when (1 month, 2 months, etc) and how much (50%, 10%, etc).
Allocations are assigned to a schedule and define who (eth address) and how much (1000 HOPR) the user is allowed to claim.
A user may have only one allocation per schedule.

The HoprDistributor smart contract should:

- on smart contract creation, it should allow us to specify HoprToken address, start time and maximum mint amount
- the owner should be able to add a schedule
- the owner should be able to add allocations
- the owner should be able to revoke a user from claiming his/hers tokens
- any user should be able to claim tokens if he/she is eligible
