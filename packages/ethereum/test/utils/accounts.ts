import { ethers } from 'ethers'

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

export const ACCOUNT_DEPLOYER = getAccount('0xf54bd518dd7e3e42710e9a96c92b1b244727df5a5afae34611089bee344d6bd4')
export const ACCOUNT_A = getAccount('0xf54bd518dd7e3e42710e9a96c92b1b244727df5a5afae34611089bee344d6bd4')
export const ACCOUNT_B = getAccount('0xf344315b0389d60ace0c8a5f36da6612d268019c2d88ff77cdb2b37f0ec7ddd5')
