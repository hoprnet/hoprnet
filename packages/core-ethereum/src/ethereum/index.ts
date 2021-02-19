import type Web3 from 'web3'
import type { AccountId } from '../types'
import { NativeBalance } from '../types'

export * from './hoprToken'

export const getNativeBalance = async (web3: Web3, account: AccountId): Promise<NativeBalance> => {
  const result = await web3.eth.getBalance(account.toHex())
  return new NativeBalance(result)
}
