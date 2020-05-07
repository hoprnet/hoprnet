import { randomBytes } from 'crypto'
import Web3 from 'web3'
import LevelUp from 'levelup'
import Memdown from 'memdown'
import { privKeyToPubKey, pubKeyToAccountId } from '.'
import CoreConnector from '..'
import { Hash, AccountId } from '../types'
import { HoprToken } from '../tsc/web3/HoprToken'
import { Await } from '../tsc/utils'

/**
 * Return private key data like public key and address
 * @param _privKey private key to derive data from
 */
export async function getPrivKeyData(_privKey: Uint8Array) {
  const privKey = new Hash(_privKey)
  const pubKey = new Hash(await privKeyToPubKey(privKey))
  const address = new AccountId(await pubKeyToAccountId(pubKey))

  return {
    privKey,
    pubKey,
    address,
  }
}

/**
 * Given web3 instance, and hoprToken instance, generate a new user and send funds to it.
 * @param web3 the web3 instance the our hoprToken contract is deployed to
 * @param funder object
 * @param hoprToken the hoprToken instance that will be used to mint tokens
 * @returns user object
 */
export async function generateUser(web3: Web3, funder: Await<ReturnType<typeof getPrivKeyData>>, hoprToken: HoprToken) {
  const user = await getPrivKeyData(randomBytes(32))

  // fund user with ETH
  await web3.eth.sendTransaction({
    value: web3.utils.toWei('1', 'ether'),
    from: funder.address.toHex(),
    to: user.address.toHex(),
  })

  // mint user some HOPR
  await hoprToken.methods.mint(user.address.toHex(), web3.utils.toWei('1', 'ether'), '0x00', '0x00').send({
    from: funder.address.toHex(),
    gas: 200e3,
  })

  return user
}

/**
 * Given a private key, generate a connector node.
 * @param privKey the private key of the connector
 * @returns CoreConnector
 */
export async function generateNode(privKey: Uint8Array): Promise<CoreConnector> {
  return CoreConnector.create(new LevelUp(Memdown()), privKey)
}
