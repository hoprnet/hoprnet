import { randomBytes } from 'crypto'
import Web3 from 'web3'
import LevelUp from 'levelup'
import Memdown from 'memdown'
import { stringToU8a } from '@hoprnet/hopr-utils'
import { privKeyToPubKey } from '.'
import CoreConnector from '..'
import { Address, Hash, Public } from '../types'
import { HoprToken } from '../tsc/web3/HoprToken'

export type Account = {
  privKey: Hash
  pubKey: Public
  address: Address
}

/**
 * Return private key data like public key and address.
 * @param _privKey private key to derive data from
 */
export async function getPrivKeyData(privKey: Uint8Array): Promise<Account> {
  const pubKey = new Public(await privKeyToPubKey(privKey))
  const address = await pubKey.toAddress()

  return {
    privKey: new Hash(privKey),
    pubKey,
    address
  }
}

/**
 * Fund an account.
 * @param web3 the web3 instance the our hoprToken contract is deployed to
 * @param hoprToken the hoprToken instance that will be used to mint tokens
 * @param funder object
 * @param account object
 */
export async function fundAccount(web3: Web3, hoprToken: HoprToken, funder: Account, account: Account) {
  // fund account with ETH
  await web3.eth.sendTransaction({
    value: web3.utils.toWei('1', 'ether'),
    from: funder.address.toHex(),
    to: account.address.toHex()
  })

  // mint account some HOPR
  await hoprToken.methods.mint(account.address.toHex(), web3.utils.toWei('1', 'ether'), '0x00', '0x00').send({
    from: funder.address.toHex(),
    gas: 200e3
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
  web3: Web3,
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

  await fundAccount(web3, hoprToken, funder, account)

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

/**
 * Disconnect web3 as if it lost connection
 * @param web3 Web3 instance
 */
export async function disconnectWeb3(web3: Web3): Promise<void> {
  try {
    // @ts-ignore
    return web3.currentProvider.disconnect(4000)
  } catch (err) {
    // console.error(err)
  }
}
