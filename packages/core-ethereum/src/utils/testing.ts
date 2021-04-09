import { randomBytes } from 'crypto'
import LevelUp from 'levelup'
import Memdown from 'memdown'
import { stringToU8a } from '@hoprnet/hopr-utils'
import CoreConnector from '..'
import { Address, Hash, PublicKey } from '../types'
import { HoprToken } from '../contracts'
import { BigNumberish, providers as IProviders, ethers, providers } from 'ethers'

export type Account = {
  privKey: Hash
  pubKey: PublicKey
  address: Address
}

/**
 * Return private key data like public key and address.
 * @param _privKey private key to derive data from
 */
export async function getPrivKeyData(privKey: Uint8Array): Promise<Account> {
  const pubKey = PublicKey.fromPrivKey(privKey)
  const address = pubKey.toAddress()

  return {
    privKey: new Hash(privKey),
    pubKey,
    address
  }
}

/**
 * Fund an account.
 * @param provider
 * @param hoprToken the hoprToken instance that will be used to mint tokens
 * @param funder object
 * @param account object
 */
export async function fundAccount(
  provider: IProviders.WebSocketProvider,
  hoprToken: HoprToken,
  funder: Account,
  account: Account
) {
  const wallet = new ethers.Wallet(account.privKey.toHex()).connect(provider)

  // fund account with ETH
  await wallet.sendTransaction({
    value: ethers.utils.parseEther('1'),
    from: funder.address.toHex(),
    to: account.address.toHex()
  })

  // mint account some HOPR
  await hoprToken
    .connect(funder.address.toHex())
    .mint(account.address.toHex(), ethers.utils.parseEther('1'), ethers.constants.HashZero, ethers.constants.HashZero, {
      gasLimit: 300e3
    })
}

/**
 * Create a random account.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export async function createAccount() {
  return getPrivKeyData(randomBytes(Hash.SIZE))
}

/**
 * Create a random account or use provided one, and then fund it.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export async function createAccountAndFund(
  provider: providers.WebSocketProvider,
  hoprToken: HoprToken,
  funder: Account,
  account?: string | Uint8Array | Account
) {
  if (typeof account === 'undefined') {
    account = await createAccount()
  } else if (typeof account === 'string') {
    account = await getPrivKeyData(stringToU8a(account))
  } else if (account instanceof Uint8Array) {
    account = await getPrivKeyData(account)
  }

  await fundAccount(provider, hoprToken, funder, account)

  return account
}

/**
 * Given a private key, create a connector node.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export async function createNode(
  privKey: Uint8Array,
  debug: boolean = true,
  maxConfirmations: number = 0
): Promise<CoreConnector> {
  return CoreConnector.create(new LevelUp(Memdown()), privKey, {
    debug,
    maxConfirmations
  })
}

// /**
//  * Disconnect web3 as if it lost connection
//  * @param web3 Web3 instance
//  */
// export async function disconnectWeb3(web3: Web3): Promise<void> {
//   try {
//     // @ts-ignore
//     return web3.currentProvider.disconnect(4000)
//   } catch (err) {
//     // console.error(err)
//   }
// }

export const advanceBlock = async (provider: IProviders.WebSocketProvider) => {
  return provider.send('evm_mine', [])
}

// increases ganache time by the passed duration in seconds
export const increaseTime = async (provider: IProviders.WebSocketProvider, _duration: BigNumberish) => {
  const duration = ethers.BigNumber.from(_duration)

  if (duration.isNegative()) throw Error(`Cannot increase time by a negative amount (${duration})`)

  await provider.send('evm_increaseTime', [duration.toNumber()])

  await advanceBlock(provider)
}

export const advanceBlockTo = async (provider: IProviders.WebSocketProvider, _target: BigNumberish) => {
  const target = ethers.BigNumber.from(_target)

  const currentBlock = await provider.getBlockNumber()
  const start = Date.now()
  let notified
  if (target.lt(currentBlock)) throw Error(`Target block #(${target}) is lower than current block #(${currentBlock})`)
  while (ethers.BigNumber.from(await provider.getBlockNumber()).lt(target)) {
    if (!notified && Date.now() - start >= 5000) {
      notified = true
    }
    await advanceBlock(provider)
  }
}
