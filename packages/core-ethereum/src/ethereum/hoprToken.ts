import type { HoprToken } from '../tsc/web3/HoprToken'
import type { Address } from '../types'
import { Balance } from '../types'
import BN from 'bn.js'

export const getBalance = async (hoprToken: HoprToken, account: Address): Promise<Balance> => {
  const result = await hoprToken.methods.balanceOf(account.toHex()).call()
  return new Balance(new BN(result))
}
