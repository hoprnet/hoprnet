#!/usr/bin/env -S yarn --silent run ts-node
// Used to fund a given address with native and ERC20 tokens up to a set target limit

import yargs from 'yargs/yargs'
import BN from 'bn.js'
import { createChainWrapper } from '@hoprnet/hopr-core-ethereum'
import { expandVars, moveDecimalPoint, Address, Balance, NativeBalance, DeferType } from '@hoprnet/hopr-utils'
import { utils } from 'ethers'
import { TX_CONFIRMATION_WAIT } from '@hoprnet/hopr-core-ethereum/src/constants'

const { PRIVATE_KEY } = process.env
const PROTOCOL_CONFIG = require('../packages/core/protocol-config.json')

function parseGasPrice(gasPrice: string) {
  const parsedGasPrice = gasPrice.split(' ')
  if (parsedGasPrice.length > 1) {
    return Number(utils.parseUnits(parsedGasPrice[0], parsedGasPrice[1]))
  }
  return Number(parsedGasPrice[0])
}

// naive mock of indexer waiting for confirmation
function createTxHandler(tx: string): DeferType<string> {
  const deferred = {} as DeferType<string>
  deferred.promise = new Promise((resolve, reject) => {
    deferred.reject = () => {
      console.log(`tx ${tx} is rejected`)
      reject(tx)
    }
    setTimeout(() => {
      deferred.resolve()
    }, TX_CONFIRMATION_WAIT)

    deferred.resolve = () => {
      console.log(`tx ${tx} is resolved`)
      resolve(tx)
    }
  })

  return deferred
}

async function getNativeBalance(chain, address: string) {
  return await chain.getNativeBalance(Address.fromString(address))
}

async function getERC20Balance(chain, address: string) {
  return await chain.getBalance(Address.fromString(address))
}

async function fundERC20(chain, sender: string, receiver: string, targetBalanceStr: string) {
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
  await chain.withdraw('HOPR', receiver, diff.toString(), (tx: string) => createTxHandler(tx))
}

async function fundNative(chain, sender: string, receiver: string, targetBalanceStr: string) {
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
  await chain.withdraw('NATIVE', receiver, diff.toString(), (tx: string) => createTxHandler(tx))
}

async function main() {
  const argv = yargs(process.argv.slice(2))
    .option('environment', {
      describe: 'environment which determines the network to run on',
      demandOption: true,
      type: 'string'
    })
    .option('address', {
      describe: 'ETH address',
      demandOption: true,
      type: 'string'
    })
    .option('erc20', {
      describe: 'whether to fund ERC20 token instead of native',
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

  const environment = PROTOCOL_CONFIG.environments[argv.environment]
  if (!environment) {
    console.error(`Cannot find environment ${environment}`)
    process.exit(1)
  }

  if (!PRIVATE_KEY) {
    console.error('Environment variable PRIVATE_KEY missing')
    process.exit(1)
  }

  // instantiate chain object based on given environment and private key
  const network = PROTOCOL_CONFIG.networks[environment.network_id]
  const chainOptions = {
    chainId: network.chain_id,
    environment: argv.environment,
    gasPrice: parseGasPrice(network.gas_price),
    network: environment.network_id,
    provider: expandVars(network.default_provider, process.env)
  }
  const privKey = utils.arrayify(PRIVATE_KEY)
  const chain = await createChainWrapper(chainOptions, privKey)
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
  .then((_result) => {
    console.log('Funding process succeeded')
  })
  .catch((err) => {
    console.log(`Error during script execution: ${err}`)
  })
