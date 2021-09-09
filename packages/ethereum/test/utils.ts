import { ethers, providers } from 'ethers'

export const getAccount = (privateKey: string) => {
  const wallet = new ethers.Wallet(privateKey)
  const uncompressedPublicKey = ethers.utils.computePublicKey(wallet.publicKey, false)

  return {
    wallet,
    privateKey: wallet.privateKey,
    uncompressedPublicKey: ethers.utils.hexDataSlice(uncompressedPublicKey, 1), // remove identifier
    publicKey: ethers.utils.hexDataSlice(wallet.publicKey, 1), // remove identifier
    address: wallet.address
  }
}

/**
 * Sign message using private key provided
 * @param message
 * @param privKey
 * @returns signature properties
 */
export const signMessage = async (message: string, privKey: string) => {
  const wallet = new ethers.Wallet(privKey)
  return {
    wallet,
    signature: await wallet.signMessage(ethers.utils.arrayify(message))
  }
}

export const toSolPercent = (multiplier: number, percent: number): string => {
  return String(Math.floor(percent * multiplier))
}

export const advanceBlock = async (provider: providers.StaticJsonRpcProvider) => {
  return provider.send('evm_mine', [])
}

// increases ganache time by the passed duration in seconds
export const increaseTime = async (provider: providers.StaticJsonRpcProvider, _duration: ethers.BigNumberish) => {
  const duration = ethers.BigNumber.from(_duration)

  if (duration.isNegative()) throw Error(`Cannot increase time by a negative amount (${duration})`)

  await provider.send('evm_increaseTime', [duration.toNumber()])

  await advanceBlock(provider)
}
