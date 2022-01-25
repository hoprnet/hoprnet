#!/usr/bin/env -S yarn --silent run ts-node
// Used to fund a given address with native and ERC20 tokens up to a set target limit

import yargs from 'yargs/yargs'
import BN from 'bn.js'
import { createChainWrapper } from '@hoprnet/hopr-core-ethereum'
import {
  expandVars,
  moveDecimalPoint,
  Address,
  Balance,
  NativeBalance,
  DeferType,
  stringToU8a
} from '@hoprnet/hopr-utils'
import { resolveEnvironment } from '@hoprnet/hopr-core'

const { PRIVATE_KEY } = process.env

type ChainWrapper = Awaited<ReturnType<typeof createChainWrapper>>

// naive mock of indexer waiting for confirmation
function createTxHandler(tx: string): DeferType<string> {
  const deferred = {} as DeferType<string>
  deferred.promise = new Promise((resolve, reject) => {
    deferred.reject = () => {
      console.log(`tx ${tx} is rejected`)
      reject(tx)
    }
    deferred.resolve = () => {
      console.log(`tx ${tx} is resolved`)
      resolve(tx)
    }
  })

  return deferred
}

async function getNativeBalance(chain: ChainWrapper, address: string) {
  return await chain.getNativeBalance(Address.fromString(address))
}

async function getERC20Balance(chain: ChainWrapper, address: string) {
  return await chain.getBalance(Address.fromString(address))
}

async function fundERC20(chain: ChainWrapper, sender: string, receiver: string, targetBalanceStr: string) {
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

async function fundNative(chain: ChainWrapper, sender: string, receiver: string, targetBalanceStr: string) {
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
    .option('erc20', {
      describe: 'if set, fund erc20 token instead of native topken',
      demandOption: false,
      type: 'boolean',
      default: false
    })
    .option('target', {
      describe: 'the target balance up to which the account shall be funded',
      demandOption: true,
      type: 'string'
    })
    .parseSync()

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
    gasPrice: environment.network.gasPrice,
    network: environment.network.id,
    provider: expandVars(environment.network.default_provider, process.env)
  }

  const privKey = stringToU8a(PRIVATE_KEY)

  // Wait as long as it takes to mine the transaction, i.e. timeout=0
  const chain = await createChainWrapper(chainOptions, privKey, true, 0)
  const sender = chain.getPublicKey().toAddress().toString()

  // maybe fund ERC20
  if (argv.erc20) {
    await fundERC20(chain, sender, argv.address, argv.target)
    return
  }
  // by default we fund native token
  await fundNative(chain, sender, argv.address, argv.target)
}

main()
  .then(() => {
    console.log('Funding process succeeded')
  })
  .catch((err) => {
    console.error(`Error during script execution: ${err}`)
    process.exit(1)
  })
