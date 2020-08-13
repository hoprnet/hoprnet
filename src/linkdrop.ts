import { PRIVATE_KEY, INFURA_PROJECT_ID, ACCOUNT_ADDRESS, CHAIN } from './env'
import LinkdropSDK from '@linkdrop/sdk'

const daiAddress = '0x5592EC0cfb4dbc12D3aB100b257153436a1f0FEa'
const factoryAddress = '0xBa051891B752ecE3670671812486fe8dd34CC1c8'

const linkdropSDK = new LinkdropSDK({
  linkdropMasterAddress: '0x' + ACCOUNT_ADDRESS,
  factoryAddress: factoryAddress,
  chain: CHAIN,
  jsonRpcUrl: `https://rinkeby.infura.io/v3/${INFURA_PROJECT_ID}`,
})

export async function payDai(amount: number, expirationHrs: number = 1.0) {
    const expirationTime = Math.round(+new Date() /1000) + (3600 * expirationHrs)
    const {
      url,
      linkId,
      linkKey,
      linkdropSignerSignature
    } = await linkdropSDK.generateLink({
      signingKeyOrWallet: PRIVATE_KEY,
      weiAmount: 0, 
      tokenAddress: daiAddress, 
      tokenAmount:  (amount * 10e17).toString(), 
      expirationTime: expirationTime,
      campaignId: 0
    })
    return url
}