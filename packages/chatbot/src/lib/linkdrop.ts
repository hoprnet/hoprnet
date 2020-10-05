import {
  LINKDROP_PRIVATE_KEY,
  LINKDROP_ACCOUNT_ADDRESS,
  LINKDROP_CHAIN,
  LINKDROP_CAMPAIGN_ID,
  LINKDROP_CAMPAIGN_AMOUNT_PER_LINK_IN_WEI,
} from '../utils/env'
import LinkdropSDK from '@linkdrop/sdk'
import TinyURL from 'tinyurl'

const factoryAddress = '0xBa051891B752ecE3670671812486fe8dd34CC1c8'

const linkdropSDK = new LinkdropSDK({
  linkdropMasterAddress: '0x' + LINKDROP_ACCOUNT_ADDRESS,
  factoryAddress: factoryAddress,
  chain: LINKDROP_CHAIN,
  jsonRpcUrl: `https://dai.poa.network`,
})

export async function payDai(expirationHrs: number = 24.0) {
  const expirationTime = Math.round(+new Date() / 1000) + 3600 * expirationHrs
  const { url } = await linkdropSDK.generateLink({
    signingKeyOrWallet: LINKDROP_PRIVATE_KEY,
    weiAmount: `${LINKDROP_CAMPAIGN_AMOUNT_PER_LINK_IN_WEI}`,
    tokenAddress: '0x0000000000000000000000000000000000000000',
    tokenAmount: 0,
    expirationTime: expirationTime,
    campaignId: LINKDROP_CAMPAIGN_ID,
  })
  return await TinyURL.shorten(url)
}
