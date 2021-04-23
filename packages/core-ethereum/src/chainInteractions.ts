import BN from 'bn.js'
import { Balance, NativeBalance } from './types'
export class ChainInteractions {
  constructor(private provider, private token) {}

  // TODO: use indexer when it's done syncing
  async getLatestBlockNumber(){
    return this.provider.getBlockNumber()
  }

  getTransactionCount(address, blockNumber){
    return this.provider.getTransactionCount(address.toHex(), blockNumber)
  }

  getBalance(address) {
    return this.token.balanceOf(address.toHex()).then((res) => new Balance(new BN(res.toString())))
  }
  
  getNativeBalance(address){
    return this.provider.getBalance(address.toHex()).then((res) => new NativeBalance(new BN(res.toString())))
  }
}
