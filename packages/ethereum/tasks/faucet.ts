import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
import type { UnsignedTransaction, BigNumber, providers } from 'ethers'
import type { HoprToken } from '@hoprnet/hopr-ethereum'

import { utils, constants } from 'ethers'
import { readdir, readFile } from 'fs/promises'
import { join } from 'path'

/**
 * Takes an array of transactions, signs them and
 * broadcasts them.
 * @param signer Used to sign transactions
 * @param txparams the transaction
 */
async function send(signer: providers.JsonRpcSigner, txparams: UnsignedTransaction): Promise<void> {
  try {
    const tx = await signer.sendTransaction(txparams)
    const txReceipt = await tx.wait()
    console.log(`Funding transaction included on-chain in block #${txReceipt.blockNumber}`)
  } catch (err) {
    console.log(`Error: ${err}`)
  }
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
  // Loading ES modules requires code changes in `hardhat-core`
  const { deserializeKeyPair, PublicKey } = await import('@hoprnet/hopr-utils')

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

  console.log(`identityFiles`, identityFiles)
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
    } else {
      console.log(`Could not decrypt private key from file ${file} using "${password}" as password (without ")`)
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

  console.log(`💧💰 Sending ${utils.formatEther(amountEth)} ETH to ${address} on network ${networkName}`)
  console.log(`💧🟡 Sending ${utils.formatEther(amountHopr)} HOPR to ${address} on network ${networkName}`)

  return txs
}

export type FaucetCLIOPts = {
  address?: string
  password: string
  useLocalIdentities: boolean
  amount: string
  identityDirectory?: string
  identityPrefix?: string
}

/**
 * Faucets HOPR and ETH tokens to a local account with HOPR
 */
async function main(
  opts: FaucetCLIOPts,
  { network, ethers, deployments, environment }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
): Promise<void> {
  const { PublicKey } = await import('@hoprnet/hopr-utils')
  if (environment == undefined) {
    console.error(`HOPR_ENVIRONMENT_ID is not set. Run with "HOPR_ENVIRONMENT_ID=<environment> ..."`)
    process.exit(1)
  }

  if (!network.tags.development) {
    console.error('Faucet is only valid in a development network')
    process.exit(1)
  }

  let hoprTokenAddress: string
  try {
    const contract = await deployments.get('HoprToken')
    hoprTokenAddress = contract.address
  } catch (error) {
    console.error('HoprToken contract has not been deployed. Deploy the contract and run again.')
    process.exit(1)
  }

  const identities: string[] = []

  if (opts.useLocalIdentities) {
    identities.push(...(await getIdentities(opts.identityDirectory, opts.password, opts.identityPrefix)))
  }

  if (opts.address) {
    if (opts.address.match(/0x[0-9a-fA-F]{40}|[0-9a-fA-F]{40}/)) {
      identities.push(opts.address)
    } else {
      try {
        identities.push(PublicKey.fromPeerIdString(opts.address).toAddress().toHex())
      } catch (err) {
        console.error(`Could not parse address ${opts.address}: ${err}`)
      }
    }
  }

  if (identities.length == 0) {
    console.error(`Could not get any usable addresses.`)
    process.exit(1)
  }

  const hoprToken = (await ethers.getContractFactory('HoprToken')).attach(hoprTokenAddress)

  // we use a custom ethers provider here instead of the ethers object from the
  // hre which is managed by hardhat-ethers, because that one seems to
  // run its own in-memory hardhat instance, which is undesirable
  const provider = new ethers.providers.JsonRpcProvider()
  const signer = provider.getSigner()

  const txs: Promise<void>[] = []
  for (const identity of identities) {
    let newTxs = await createTransaction(
      hoprToken,
      identity,
      utils.parseEther('10.0'),
      utils.parseEther('20000.0'),
      network.name
    )
    for (const tx of newTxs) {
      let txSend = send(signer, tx)
      txs.push(txSend)
    }
  }

  await Promise.all(txs)
}

export default main
