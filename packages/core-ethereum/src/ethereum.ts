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
  const result = await token.balanceOf(account.toHex())
  return new Balance(new BN(result.toString()))
}

export function createChainWrapper(provider, token) {
  return {
    // TODO: use indexer when it's done syncing
    getLatestBlockNumber: async () => provider.getBlockNumber(),
    getTransactionCount: (address, blockNumber) => provider.getTransactionCount(address.toHex(), blockNumber),
    getBalance: (address) => token.balanceOf(address.toHex()).then((res) => new Balance(new BN(res.toString()))),
    getNativeBalance: (address) =>
      provider.getBalance(address.toHex()).then((res) => new NativeBalance(new BN(res.toString())))
  }
}
