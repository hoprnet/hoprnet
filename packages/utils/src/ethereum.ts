import { errors } from 'ethers'

// These functions allow us to differentiate between error messages from
// interacting with the chain.

export function isErrorOutOfNativeFunds(error: any): boolean {
  // https://github.com/ethers-io/ethers.js/blob/bde861436ebef572d04ae8a7a111b8b954b4571c/packages/providers/src.ts/json-rpc-provider.ts#L52
  return [error?.code, String(error)].includes(errors.INSUFFICIENT_FUNDS)
}

export function isErrorOutOfHoprFunds(error: any): boolean {
  // we assume all `SafeMath: subtraction overflow` are due to lack of funds
  // https://github.com/ethers-io/ethers.js/blob/d3b7130ed6ec50b192eb7f33905eaa327d65eee2/packages/logger/src.ts/index.ts#L123
  return [error?.reason, String(error)].includes('SafeMath: subtraction overflow')
}

export function isErrorOutOfFunds(error: any): 'NATIVE' | 'HOPR' | false {
  if (isErrorOutOfNativeFunds(error)) return 'NATIVE'
  else if (isErrorOutOfHoprFunds(error)) return 'HOPR'
  return false
}
