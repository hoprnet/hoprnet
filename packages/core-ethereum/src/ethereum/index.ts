import type Web3 from 'web3'
import type { Address } from '../types'
import BN from 'bn.js'
import { NativeBalance } from '../types'

export * from './hoprToken'

export const getNativeBalance = async (web3: Web3, account: Address): Promise<NativeBalance> => {
  const result = await web3.eth.getBalance(account.toHex())
  return new NativeBalance(new BN(result))
}
