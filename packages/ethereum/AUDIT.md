# Scope

To be audited:

- [HoprToken.sol](./contracts/HoprToken.sol)
- [ERC777Snapshot.sol](./contracts/ERC777/ERC777Snapshot.sol)
- [HoprDistributor.sol](./contracts/HoprDistributor.sol)

# Scope of Audit

These smart contracts are expected to run on the current Ethereum main chain as described below.

## HoprToken

[[source code](./contracts/HoprToken.sol)]

HoprToken is an ERC777 and ERC20 compatible token, it also allows for querrying balance data at a specific block. To achieve that we have [extended](./contracts/ERC777/ERC777Snapshot.sol) OZ's ERC777 contract and implemented a snapshotting mechanism similar to the [minime token](https://github.com/Giveth/minime/blob/ea04d950eea153a04c51fa510b068b9dded390cb/contracts/MiniMeToken.sol).

The HoprToken smart contract should:

- be ERC20 compatible
- be ERC777 compatible
- allow a user with role "MINTER_ROLE" to be able to mint tokens
- allow multiple accounts to be minter at the same time
- have an admin account who can appoint appoint minters
- allow admin to transfer or revokable their ownership via OZ `AccessControl` contract
- expose snapshot functionality, specifically it should
  - expose a function `getBalanceOfAt` to return the balance of a user at specified block number, if the account transferred tokens then the balance at the end of the block is taken. This number is returned for all following blocks until another transfer to or from that account happened. This should allow for snapshot-based governance such as snapshot.page (which currently works with an archive node but an on-chain snapshot read strategy might be implemented in the future).
  - expose a function `getTotalSupplyAt` to return the totalSupply at specified block number, same mechanism as for balance

## HoprDistributor

[[source code](./contracts/HoprDistributor.sol)]

HoprDistributor is a smart contract that allows distribution of HOPR tokens for testnet participants (bounties), early token purchasers, advisors and team. It allows for creation schedules and allocations, so that then an eligible user who can claim their tokens. Schedules are named and comprise the release time relative to the start time (1 month, 2 months, etc) and how much (50%, 10%, etc). Allocations are assigned to a schedule and define who (eth address) and how much (1000 HOPR) the user is allowed to claim. A user may have only one allocation per schedule.

The HoprDistributor smart contract should:

- allow deployer to specify HoprToken address, start time and maximum mint amount on smart contract creation
- allow the owner to specify a different start time until the previously defined start time has been reached
- [schedule]
  - allow the owner to add a schedule
  - not accept the same schedule name more than once
  - not accept invalid schedules with different array lengths
  - not accept invalid schedules with durations that are not in ascending order
  - not allow schedules which in total distribute more than 100% of their allocation
- [allocation]
  - allow the owner should to add allocations
  - not allow for allocations to be added to schedules that do not exist
  - not accept invalid allocations with different array lengths
  - not allow the owner to
  - not allow the owner to add allocations that exceed the total amount of mintable tokens
  - allow any user to claim tokens if they are eligible for the specified schedule at that time
  - allow any account to claim tokens for an eligible account so that the eligible account does not need to send the transaction or pay for gas themselves
- [revoke]
  - allow the owner to revoke a user from claiming their tokens
  - prevent an account to claim any tokens from the time that their allocation has been revoked, this includes tokens which were already due to be released but that users has not claimed yet - for sake of clarity such tokens should also not be claimable anymore
  - not allow the owner to revoke allocations or schedules that do not exist
