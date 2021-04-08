import type { providers as IProviders } from 'ethers'
import type { Address } from './types'
import type { HoprToken } from './contracts'
import BN from 'bn.js'
import { Balance, NativeBalance } from './types'

export const getNativeBalance = async (
  provider: IProviders.WebSocketProvider,
  account: Address
): Promise<NativeBalance> => {
  const result = await provider.getBalance(account.toHex())
  return new NativeBalance(new BN(result.toString()))
}

export const getBalance = async (token: HoprToken, account: Address): Promise<Balance> => {
  const result = await token.methods.balanceOf(account.toHex()).call()
  return new Balance(new BN(result))
}
