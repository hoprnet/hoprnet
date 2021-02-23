import type { HoprToken } from '../tsc/web3/HoprToken'
import type { AccountId } from '../types'
import { Balance } from '../types'

export const getBalance = async (hoprToken: HoprToken, account: AccountId): Promise<Balance> => {
  const result = await hoprToken.methods.balanceOf(account.toHex()).call()
  return new Balance(result)
}
