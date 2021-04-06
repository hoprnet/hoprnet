import Ethers from 'ethers'

const { utils } = Ethers

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

/**
 * Prefix message with our special message
 * @param message
 * @returns hashed message
 */
export const prefixMessageWithHOPR = (message: string) => {
  const withHOPR = utils.hexConcat([utils.hexlify('HOPRnet'), message])

  return utils.solidityKeccak256(
    ['string', 'string', 'bytes'],
    ['\x19Ethereum Signed Message:\n', utils.hexDataLength(withHOPR), withHOPR]
  )
}

/**
 * Sign message using private key provided
 * @param message
 * @param privKey
 * @returns signature properties
 */
export const signMessage = async (message: string, privKey: string) => {
  const wallet = new Ethers.Wallet(privKey)
  const signature = await wallet.signMessage(message)
  const { r, s, v } = utils.splitSignature(signature)

  return {
    message,
    signature,
    r,
    s,
    v
  }
}
