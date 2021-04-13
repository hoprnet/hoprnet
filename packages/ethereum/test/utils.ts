import { ethers, providers } from 'ethers'

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
    ['\x19Ethereum Signed Message:\n', withHOPR.length.toString(), withHOPR]
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

export const toSolPercent = (multiplier: number, percent: number): string => {
  return String(Math.floor(percent * multiplier))
}

export const advanceBlock = async (provider: providers.JsonRpcProvider) => {
  return provider.send('evm_mine', [])
}

// increases ganache time by the passed duration in seconds
export const increaseTime = async (provider: providers.JsonRpcProvider, _duration: ethers.BigNumberish) => {
  const duration = ethers.BigNumber.from(_duration)

  if (duration.isNegative()) throw Error(`Cannot increase time by a negative amount (${duration})`)

  await provider.send('evm_increaseTime', [duration.toNumber()])

  await advanceBlock(provider)
}
