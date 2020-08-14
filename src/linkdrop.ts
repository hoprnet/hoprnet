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
  jsonRpcUrl: `https://rinkeby.infura.io/v3/${INFURA_PROJECT_ID}`,
})

const fees = 2000000000000000

export async function setupPayDai(amount) {
  const proxyAddress = linkdropSDK.getProxyAddress(CAMPAIGN_ID)
  await linkdropSDK.approve({ 
      signingKeyOrWallet: PRIVATE_KEY,
      proxyAddress: proxyAddress,
      tokenAddress: '0x' + DAI_ADDRESS,
      tokenAmount: (amount * 10e17).toString()
  })
}

export async function payDai(amount: number, expirationHrs: number = 1.0) {
    const proxyAddress = linkdropSDK.getProxyAddress(CAMPAIGN_ID)
    const txHash = await linkdropSDK.topup({ 
        signingKeyOrWallet : PRIVATE_KEY,
        proxyAddress: proxyAddress,
        weiAmount: fees, 
    })

    const expirationTime = Math.round(+new Date() /1000) + (3600 * expirationHrs)
    const {
      url,
      linkId,
      linkKey,
      linkdropSignerSignature
    } = await linkdropSDK.generateLink({
      signingKeyOrWallet: PRIVATE_KEY,
      weiAmount: 0, 
      tokenAddress: '0x' + DAI_ADDRESS, 
      tokenAmount:  (amount * 10e17).toString(), 
      expirationTime: expirationTime,
      campaignId: CAMPAIGN_ID
    })
    return await TinyURL.shorten(url)
}