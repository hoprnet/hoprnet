# Mainnet deployment

A step by step guide on how to deploy [HoprToken](../contracts/HoprToken.sol) and [HoprDistributor](../contracts/HoprDistributor.sol) on mainnet and then afterwards transferring all admin roles to a multisig.

## High-level transactions

deployer = private key within `.env`

1. deploy `HoprToken` using `deployer`
2. deploy `HoprDistributor` using `deployer`
3. addSchedule using `HoprDistributor`
4. addAllocations using `HoprDistributor`
5. grantMinterRole using `HoprToken` to `HoprDistributor`
6. grantAdminRole using `HoprToken` to `multisig`
7. renounceAdmin of `deployer` using `HoprToken`
8. transferOwner of `HoprDistributor` to `multisig`

## Setting up repository

Install & Setup our monorepo, in project root directory, run the following commands in this order:

1. `yarn`
2. `yarn setup`
3. `yarn build`

## Setting up private key

In order to sign transactions we will have to setup our private key and infura key:

1.  `cd ./packages/ethereum`
2.  `cp .env.example .env`
3.  set `PRIVATE_KEY` and `INFURA` values

## Deploying HoprToken and HoprDistributor

1. `npx hardhat migrate-mainnet --task deployToken --network mainnet`
2. `npx hardhat migrate-mainnet --task deployDist --network mainnet`

You may specify `--network xdai` for a deployment to xDAI.
Both of this commands will update the [addresses file](../chain/addresses.json) with the deployed contracts' addresses.

## Preparing HoprDistributor schedules and allocations

The files containing all data to use for contracts constructors / method calls should already be included
in the [data folder](./data).

## Setting up HoprDistributor

1. `npx hardhat migrate-mainnet --task addSchedule --schedule bounties --network mainnet`
2. `npx hardhat migrate-mainnet --task addAllocations --allocation bounties --network mainnet`
3. `npx hardhat migrate-mainnet --task grantMinter --network mainnet`

## Transfering administrative roles to the multisig

1. `npx hardhat migrate-mainnet --task grantTokenAdmin --network mainnet`
2. `npx hardhat migrate-mainnet --task renounceTokenAdmin --network mainnet`
3. `npx hardhat migrate-mainnet --task transferDistOwner --network mainnet`
