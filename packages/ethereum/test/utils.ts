import { ethers } from 'ethers'

/**
 * Depending on what network tests are run, the error output
 * may vary. This utility prefixes the error to it matches
 * with hardhat's network where our tests run.
 * @param error
 * @returns error prefixed by network's message
 */
export const vmErrorMessage = (error: string) => {
  return `VM Exception while processing transaction: revert ${error}`
}

export const getAccount = (privateKey: string) => {
  const wallet = new ethers.Wallet(privateKey)
  const uncompressedPublicKey = ethers.utils.computePublicKey(wallet.publicKey, false)

  return {
    privateKey: wallet.privateKey,
    uncompressedPublicKey: ethers.utils.hexDataSlice(uncompressedPublicKey, 1), // remove identifier
    publicKey: ethers.utils.hexDataSlice(wallet.publicKey, 1), // remove identifier
    address: wallet.address
  }
}

/**
 * Prefix message with our special message
 * @param message
 * @returns hashed message
 */
export const prefixMessageWithHOPR = (message: string) => {
  const withHOPR = ethers.utils.concat([ethers.utils.toUtf8Bytes('HOPRnet'), message])

  return ethers.utils.solidityKeccak256(
    ['string', 'string', 'bytes'],
    ['\x19Ethereum Signed Message:\n', withHOPR.length, withHOPR]
  )
}

/**
 * Sign message using private key provided
 * @param message
 * @param privKey
 * @returns signature properties
 */
export const signMessage = async (message: string, privKey: string) => {
  const wallet = new ethers.Wallet(privKey)
  // const signature = await wallet.signMessage(message)
  // we do not use above since we use a different prefix
  const signature = ethers.utils.joinSignature(wallet._signingKey().signDigest(message))
  const { r, s, v } = ethers.utils.splitSignature(signature)

  return {
    message,
    signature,
    r,
    s,
    v
  }
}
