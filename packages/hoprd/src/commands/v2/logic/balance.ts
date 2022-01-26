import Hopr from '@hoprnet/hopr-core'
import { isError } from '..'

export const getBalances = async (node: Hopr) => {
  const hoprBalance = (await node.getBalance()).toFormattedString()
  const nativeBalance = (await node.getNativeBalance()).toFormattedString()
  const err = isError(hoprBalance) || isError(nativeBalance)
  return err ? new Error('failure') : { native: nativeBalance, hopr: hoprBalance }
}
