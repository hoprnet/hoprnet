import { PRIVATE_KEY } from './env'
import LinkdropSDK from '@linkdrop/sdk'

const linkdropSDK = new LinkdropSDK({
  linkdropMasterAddress: '0x1A59A353533FEff03528520704FF34C115761203',
  factoryAddress: '0xBa051891B752ecE3670671812486fe8dd34CC1c8',
  chain: 'rinkeby',
  jsonRpcUrl: 'https://rinkeby.infura.io/v3/37bcabbbefb548b69bb8c4e841fd86b0',
})

export async function pay() {
    console.log('0x' + PRIVATE_KEY)
    const {
      url,
      linkId,
      linkKey,
      linkdropSignerSignature
    } = await linkdropSDK.generateLink({
      signingKeyOrWallet: '0x' + PRIVATE_KEY,
      weiAmount: 0, 
      tokenAddress: '0x5592EC0cfb4dbc12D3aB100b257153436a1f0FEa', 
      tokenAmount: 30000000000000000000, 
      expirationTime: 12345678910, 
      campaignId: 0,
      projectId: 0
    })
    console.log(url)
}