import PeerId from 'peer-id'
import Web3 from 'web3'
import { sha3, toChecksumAddress } from 'web3-utils'
import { publicKeyConvert } from 'secp256k1'

const COMPRESSED_PUBLIC_KEY_LENGTH = 33

/**
 * Signs a transaction with the private key that is given by
 * the peerId instance and publishes it to the network given by
 * the web3.js instance
 *
 * @param tx an Ethereum transaction
 * @param peerId a peerId
 * @param web3 a web3.js instance
 */
export async function sendTransaction(tx: any, peerId: PeerId, web3: Web3): Promise<any> {
  const signedTransaction = await signTransaction(tx, peerId, web3)

  return web3.eth.sendSignedTransaction(signedTransaction.rawTransaction).then(receipt => {
    if (typeof receipt.status === 'string') {
      receipt.status = Boolean(parseInt(receipt.status, 16))
    }

    if (typeof receipt.status === 'number') {
      receipt.status === Boolean(receipt.status)
    }

    if (!receipt.status) throw Error('Reverted tx.')

    return receipt
  })
}

export async function signTransaction(tx: any, peerId: PeerId, web3: Web3): Promise<any> {
  const account = peerIdToWeb3Account(peerId, web3)

  return account.signTransaction(
    Object.assign(tx, {
      from: pubKeyToEthereumAddress(peerId.pubKey.marshal()),
      gasPrice: await web3.eth.getGasPrice()
    })
  )
}

/**
 * Creates a web3 account from a peerId instance.
 *
 * @param peerId a peerId instance
 * @param web3 a web3.js instance
 */
export function peerIdToWeb3Account(peerId: PeerId, web3: Web3): any {
  if (!peerId.privKey)
    throw Error(`Unable to find private key. Please insert a peerId that is equipped with a private key.`)

  return web3.eth.accounts.privateKeyToAccount('0x'.concat(peerId.privKey.marshal().toString('hex')))
}

/**
 * Derives an Ethereum address from a given public key.
 *
 * @param pubKey given as compressed elliptic curve point.
 *
 * @returns e.g. 0xc1912fEE45d61C87Cc5EA59DaE31190FFFFf232d
 */
export function pubKeyToEthereumAddress(pubKey: Buffer) {
  if (pubKey.length != COMPRESSED_PUBLIC_KEY_LENGTH)
    throw Error(
      `Invalid input parameter. Expected a Buffer of size ${COMPRESSED_PUBLIC_KEY_LENGTH}. Got '${typeof pubKey}'${
        pubKey.length ? ` of length ${pubKey.length}` : ''
      }.`
    )

  const hash = sha3(
    publicKeyConvert(pubKey, false)
      .slice(1)
      .toString('hex')
  )

  return toChecksumAddress(hash.replace(/(0x)[0-9a-fA-F]{24}([0-9a-fA-F]{20})/, '$1$2'))
}

/**
 * Checks whether the ethereum address of the @param sender is
 * smaller than the ethereum address of the @param otherParty
 *
 * @param sender an ethereum address
 * @param otherParty another ethereum address
 */
export function isPartyA(sender: string | Buffer, otherParty: string | Buffer): boolean {
  if (typeof sender === 'string') {
    if (sender.length !== 42) throw Error('Invalid input parameters')

    sender = Buffer.from(sender.replace(/0x/, ''), 'hex')
  }

  if (typeof otherParty === 'string') {
    if (otherParty.length !== 42) {
      throw Error('Invalid input parameters')
    }
    otherParty = Buffer.from(otherParty.replace(/0x/, ''), 'hex')
  }

  if (!Buffer.isBuffer(sender) || !Buffer.isBuffer(otherParty)) throw Error('Invalid input parameters')

  if (sender.length != 20 || otherParty.length != 20) throw Error('Invalid input parameters')

  return Buffer.compare(sender, otherParty) < 0
}

const ETHEUREUM_ADDRESS_SIZE = 20 // Bytes

/**
 * Computes the ID that is used by the smart contract to
 * store payment channels.
 *
 * @param sender an ethereum address or the corresponding public key
 * @param counterparty another ethereum address or the corresponding public key
 * @returns the Id
 */
export function getId(sender: string | Buffer, counterparty: string | Buffer): Buffer {
  if (Buffer.isBuffer(sender) && sender.length == COMPRESSED_PUBLIC_KEY_LENGTH) {
    sender = pubKeyToEthereumAddress(sender)
  }

  if (Buffer.isBuffer(counterparty) && counterparty.length == COMPRESSED_PUBLIC_KEY_LENGTH) {
    counterparty = pubKeyToEthereumAddress(counterparty)
  }

  if (typeof sender !== 'string' || typeof counterparty !== 'string')
    throw Error(
      `Invalid input parameters. Unable to convert ${typeof sender} and / or ${typeof counterparty} to an Ethereum address.`
    )

  sender = Buffer.from(sender.replace(/0x/, ''), 'hex')
  counterparty = Buffer.from(counterparty.replace(/0x/, ''), 'hex')

  if (module.exports.isPartyA(sender, counterparty)) {
    return module.exports.hash(Buffer.concat([sender, counterparty], 2 * ETHEUREUM_ADDRESS_SIZE))
  } else {
    return module.exports.hash(Buffer.concat([counterparty, sender], 2 * ETHEUREUM_ADDRESS_SIZE))
  }
}
