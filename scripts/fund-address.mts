#!/usr/bin/env -S yarn --silent run ts-node
// Used to fund a given address with native and ERC20 tokens up to a set target limit

import yargs from 'yargs/yargs'
import BN from 'bn.js'
import {
  expandVars,
  moveDecimalPoint,
  Address,
  Balance,
  NativeBalance,
  DeferType,
  stringToU8a
} from '@hoprnet/hopr-utils'

const { PRIVATE_KEY } = process.env

// naive mock of indexer waiting for confirmation
function createTxHandler(tx: string): DeferType<string> {
  const state = {
    promise: Promise.resolve(),
    reject: () => {
      console.log(`tx ${tx} got rejected`)
      state.promise = Promise.reject()
    }
  }

  // Don't need to implement `resolve` method
  // because it is intended to be called by the
  // indexer - which we don't use in this script, so
  // we must only ensure that we reject once there is an error
  return state as unknown as DeferType<string>
}

// @fixme fix ESM issue and use ChainWrapper type
async function getNativeBalance(chain: any, address: string) {
  return await chain.getNativeBalance(Address.fromString(address))
}

// @fixme fix ESM issue and use ChainWrapper type
async function getERC20Balance(chain: any, address: string) {
  return await chain.getBalance(Address.fromString(address))
}

// @fixme fix ESM issue and use ChainWrapper type
async function fundERC20(chain: any, sender: string, receiver: string, targetBalanceStr: string) {
  const senderBalance = await getNativeBalance(chain, sender)
  const balance = await getERC20Balance(chain, receiver)
  const targetBalanceNr = moveDecimalPoint(targetBalanceStr, Balance.DECIMALS)
  const targetBalance = new Balance(new BN(targetBalanceNr))
  const diff = targetBalance.sub(balance)

  // stop if sender doesn't have enough funds
  if (senderBalance.lte(diff)) {
    console.log(
      `sender ${sender} has ${senderBalance.toFormattedString()}, not enough to fund ${diff.toFormattedString()} to receiver ${receiver}`
    )
    return
  }

  // stop if receiver has enough funds already
  if (diff.lte(new Balance(new BN(0)))) {
    console.log(
      `receiver ${receiver} has ${balance.toFormattedString()}, ${diff.toFormattedString()} more than defined target ${targetBalance.toFormattedString()}`
    )
    return
  }

  // fund the difference
  console.log(
    `transfer ${diff.toFormattedString()} from ${sender} to ${receiver} to top up ${balance.toFormattedString()}`
  )
  await chain.withdraw('HOPR', receiver, diff.toString(), createTxHandler)
}

// @fixme fix ESM issue and use ChainWrapper type
async function fundNative(chain: any, sender: string, receiver: string, targetBalanceStr: string) {
  const senderBalance = await getNativeBalance(chain, sender)
  const balance = await getNativeBalance(chain, receiver)
  const targetBalanceNr = moveDecimalPoint(targetBalanceStr, Balance.DECIMALS)
  const targetBalance = new NativeBalance(new BN(targetBalanceNr))
  const diff = targetBalance.sub(balance)

  // stop if sender doesn't have enough funds
  if (senderBalance.lte(diff)) {
    console.log(
      `sender ${sender} has ${senderBalance.toFormattedString()}, not enough to fund ${diff.toFormattedString()} to receiver ${receiver}`
    )
    return
  }

  // stop if receiver has enough funds already
  if (diff.lte(new NativeBalance(new BN(0)))) {
    console.log(
      `receiver ${receiver} has ${balance.toFormattedString()}, ${diff.toFormattedString()} more than defined target ${targetBalance.toFormattedString()}`
    )
    return
  }

  // fund the difference
  console.log(
    `transfer ${diff.toFormattedString()} from ${sender} to ${receiver} to top up ${balance.toFormattedString()}`
  )
  await chain.withdraw('NATIVE', receiver, diff.toString(), createTxHandler)
}

async function main() {
  const argv = yargs(process.argv.slice(2))
    .option('environment', {
      describe: 'environment which determines the network to run on',
      demandOption: true,
      type: 'string'
    })
    .option('address', {
      describe: 'ETH address of the recipient',
      demandOption: true,
      type: 'string'
    })
    .option('targetERC20', {
      describe: 'the target balance up to which the account shall be funded',
      type: 'string'
    })
    .option('targetNative', {
      describe: 'the target balance up to which the account shall be funded',
      type: 'string'
    })
    .parseSync()

  if (!argv.targetERC20 && !argv.targetNative) {
    console.error(`Running fund script without a fund option.`)
    process.exit(1)
  }

  const { resolveEnvironment } = await import('@hoprnet/hopr-core')
  const environment = resolveEnvironment(argv.environment)
  if (!environment) {
    console.error(`Cannot find environment ${environment}`)
    process.exit(1)
  }

  if (!PRIVATE_KEY) {
    console.error('Environment variable PRIVATE_KEY missing')
    process.exit(1)
  }

  // instantiate chain object based on given environment and private key
  const chainOptions = {
    chainId: environment.network.chain_id,
    environment: environment.id,
    maxFeePerGas: environment.network.max_fee_per_gas,
    maxPriorityFeePerGas: environment.network.max_priority_fee_per_gas,
    network: environment.network.id,
    provider: expandVars(environment.network.default_provider, process.env)
  }

  const privKey = stringToU8a(PRIVATE_KEY)

  const { createChainWrapper } = await import('@hoprnet/hopr-core-ethereum')
  const { getContractData } = await import('@hoprnet/hopr-core/src/environment.js')

  // Wait as long as it takes to mine the transaction, i.e. timeout=0
  const deploymentExtract = getContractData(environment.id)
  const chain = await createChainWrapper(deploymentExtract, chainOptions, privKey, true, 0)
  const sender = chain.getPublicKey().toAddress().toString()

  // maybe fund ERC20
  if (argv.targetERC20) {
    await fundERC20(chain, sender, argv.address, argv.targetERC20)
  }

  if (argv.targetNative) {
    await fundNative(chain, sender, argv.address, argv.targetNative)
  }
}

main()
  .then(() => {
    console.log('Funding process succeeded')
  })
  .catch((err) => {
    console.error(`Error during script execution: ${err}`)
    process.exit(1)
  })
