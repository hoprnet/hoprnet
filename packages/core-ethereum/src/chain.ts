import { ChainWrapper, createChainWrapper } from './ethereum'
import { debug } from '@hoprnet/hopr-utils'

const log = debug('hopr:core-ethereum:chain')

export default class ChainWrapperSingleton {
  private static instance: ChainWrapper
  private constructor() {}
  public static async create(
    networkInfo: { provider: string; chainId: number; gasPrice?: number; network: string; environment: string },
    privateKey: Uint8Array,
    checkDuplicate: Boolean = true
  ): Promise<ChainWrapper> {
    log('Receiving create request for hopr-ethereum with `networkInfo`', networkInfo)
    if (!ChainWrapperSingleton.instance) {
      log('Non-existant singleton instance, creating for the first time.')
      ChainWrapperSingleton.instance = await createChainWrapper(networkInfo, privateKey, checkDuplicate)
    }
    return ChainWrapperSingleton.instance
  }
}
