import { 
  PRIVATE_KEY, 
  INFURA_PROJECT_ID, 
  ACCOUNT_ADDRESS, 
  CHAIN, DAI_ADDRESS, 
  CAMPAIGN_ID 
} from './env'
import LinkdropSDK from '@linkdrop/sdk'
import TinyURL from 'tinyurl'

const factoryAddress = '0xBa051891B752ecE3670671812486fe8dd34CC1c8'

const linkdropSDK = new LinkdropSDK({
  linkdropMasterAddress: '0x' + ACCOUNT_ADDRESS,
  factoryAddress: factoryAddress,
  chain: CHAIN,
  jsonRpcUrl: `https://dai.poa.network`,
})

const fees = 2000000000000000

//@TODO Only call setup setupPayDai if DAI is actually needed
// export async function setupPayDai(amount) {
//   const proxyAddress = linkdropSDK.getProxyAddress(CAMPAIGN_ID)
//   await linkdropSDK.approve({ 
//       signingKeyOrWallet: PRIVATE_KEY,
//       proxyAddress: proxyAddress,
//       tokenAddress: '0x0000000000000000000000000000000000000000',
//       tokenAmount: (amount * 10e17).toString()
//   })
// }

export async function payDai(amount: number, expirationHrs: number = 24.0) {
     // @TODO Call proxy address only if needed
    // const proxyAddress = linkdropSDK.getProxyAddress(CAMPAIGN_ID)
    // const txHash = await linkdropSDK.topup({ 
    //     signingKeyOrWallet : PRIVATE_KEY,
    //     proxyAddress: proxyAddress,
    //     weiAmount: fees, 
    // })
    // */

    const expirationTime = Math.round(+new Date() /1000) + (3600 * expirationHrs)
    const {
      url,
      linkId,
      linkKey,
      linkdropSignerSignature
    } = await linkdropSDK.generateLink({
      // @TODO Remove Private Key if we using manual campaign
      signingKeyOrWallet: PRIVATE_KEY,
      // signingKeyOrWallet: '0x44f42bf8a133aac7dbf50baf1d5dd67ebad64a376330c058029aaa04c688e465',
      weiAmount: "10000000000000000",
      tokenAddress: '0x0000000000000000000000000000000000000000',
      //@TODO Restore tokenAddress for DAI
      // tokenAddress: '0x' + DAI_ADDRESS,
      tokenAmount: 0,
      expirationTime: expirationTime,
      campaignId: 3
    })
    return await TinyURL.shorten(url)
}