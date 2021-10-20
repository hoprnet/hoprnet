import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import type { UnsignedTransaction, BigNumber, providers } from 'ethers'
import type { HoprToken } from '../types'

import { utils, constants } from 'ethers'
import { deserializeKeyPair, PublicKey, hasB58String } from '@hoprnet/hopr-utils'
import { getContractData, Networks } from '..'
import { readdir, readFile } from 'fs/promises'
import { join } from 'path'

/**
 * Takes an array of transactions, signs them and
 * broadcasts them.
 * @param signer Used to sign transactions
 * @param txparams the transaction
 */
async function send(signer: providers.JsonRpcSigner, txparams: UnsignedTransaction): Promise<void> {
  let txResponse: providers.TransactionResponse
  try {
    txResponse = await signer.sendTransaction(txparams)
  } catch (err) {
    console.log(`Error: ${err}`)
  }

  console.log(`transactionHash: ${txResponse.hash}`)
}
/**
 * Reads the identity files from the given directory, decrypts
 * them and returns their Ethereum addresses
 * @param directory directory to look for identity files
 * @param password password to decrypt identity files
 * @param prefix only take identities with given prefix
 * @returns the identities' Ethereum addresses
 */
async function getIdentities(directory: string, password: string, prefix?: string): Promise<string[]> {
  let fileNames: string[]
  try {
    fileNames = await readdir(directory)
  } catch (err) {
    console.log(err)
    return []
  }

  const identityFiles: string[] = []
  for (const fileName of fileNames) {
    if ((prefix == null || fileName.startsWith(prefix)) && fileName.endsWith('.id')) {
      identityFiles.push(fileName)
    }
  }

  const identites: string[] = []
  for (const identityFile of identityFiles) {
    let file: Uint8Array
    const path = join(directory, identityFile)
    try {
      file = await readFile(path)
    } catch (err) {
      console.log(`Could not access ${path}.`, err)
      continue
    }

    const decoded = await deserializeKeyPair(file, password, true)

    if (decoded.success) {
      identites.push(PublicKey.fromPeerId(decoded.identity).toAddress().toHex())
    }
  }

  return identites
}

/**
 * Creates two transaction: one that sends ETH to the address
 * and a second one which sends HOPR tokens to that address
 * @param token instance of the HOPR token
 * @param address where to send the HOPR tokens and ETH to
 * @param amountEth how many ETH
 * @param amountHopr how many HOPR
 * @param networkName which network to use
 * @returns
 */
async function createTransaction(
  token: HoprToken,
  address: string,
  amountEth: BigNumber,
  amountHopr: BigNumber,
  networkName: string
): Promise<UnsignedTransaction[]> {
  const txs: UnsignedTransaction[] = [
    await token.populateTransaction.mint(address.toString(), amountHopr, constants.HashZero, constants.HashZero, {
      gasLimit: 200e3
    }),
    {
      to: address,
      value: amountEth
    }
  ]

  console.log(`💧💰 Sending ${amountEth.toString()} ETH to ${address} on network ${networkName}`)
  console.log(`💧🟡 Sending ${amountHopr.toString()} HOPR to ${address} on network ${networkName}`)

  return txs
}

type CLIOPts = {
  address?: string
  useLocalIdentities: boolean
  amount: string
  identityDirectory?: string
  identityPrefix?: string
}

/**
 * Faucets HOPR and ETH tokens to a local account with HOPR
 */
async function main(opts: CLIOPts, { ethers, network }: HardhatRuntimeEnvironment, _runSuper: RunSuperFunction<any>) {
  if (network.tags.development) {
    console.error('🌵 Faucet is only valid in a development network')
    return
  }

  let hoprTokenAddress: string
  try {
    const contract = getContractData(network.name as Networks, 'HoprToken')
    hoprTokenAddress = contract.address
  } catch {
    console.error('⛓  You need to ensure the network deployed the contracts')
    return
  }

  const identities: string[] = []

  if (opts.useLocalIdentities) {
    identities.push(...(await getIdentities(opts.identityDirectory, 'e2e-test', opts.identityPrefix)))
  }

  if (opts.address) {
    if (opts.address.match(/0x[0-9a-fA-F]{40}|[0-9a-fA-F]{40}/)) {
      identities.push(opts.address)
    } else if (hasB58String(opts.address)) {
      try {
        identities.push(PublicKey.fromPeerIdString(opts.address).toAddress().toHex())
      } catch (err) {
        console.log(`error while parsing ${opts.address}`)
      }
    } else {
      console.log(`Address ${opts.address} has unknown format.`)
    }
  }

  if (identities.length == 0) {
    throw Error(`Could not get any usable addresses.`)
  }

  const signer = ethers.provider.getSigner()
  const hoprToken = (await ethers.getContractFactory('HoprToken')).attach(hoprTokenAddress) as HoprToken

  const finalAmount = utils.parseEther('1.0')

  const txs = await createTransaction(hoprToken, identities[0], finalAmount, finalAmount, network.name)

  await Promise.all(txs.map((tx) => send(signer, tx)))
}

export default main
