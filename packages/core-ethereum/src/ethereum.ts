import type Web3 from 'web3'
import type { Address } from './types'
import type { HoprToken } from './tsc/web3/HoprToken'
import BN from 'bn.js'
import { Balance, NativeBalance } from './types'

export const getNativeBalance = async (web3: Web3, account: Address): Promise<NativeBalance> => {
  const result = await web3.eth.getBalance(account.toHex())
  return new NativeBalance(new BN(result))
}

export const getBalance = async (token: HoprToken, account: Address): Promise<Balance> => {
  const result = await token.methods.balanceOf(account.toHex()).call()
  return new Balance(new BN(result))
}
