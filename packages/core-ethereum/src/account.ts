import type { Wallet as IWallet } from 'ethers'
import BN from 'bn.js'
import { ethers } from 'ethers'
import { isExpired } from '@hoprnet/hopr-utils'
import { PublicKey, Address, Balance, Hash, NativeBalance, UINT256 } from './types'
import Indexer from './indexer'
import { PROVIDER_CACHE_TTL } from './constants'
import type { ChainWrapper } from './ethereum'

export const EMPTY_HASHED_SECRET = new Hash(ethers.utils.arrayify(ethers.constants.HashZero))
const cache = new Map<'balance' | 'nativeBalance', { value: string; updatedAt: number }>()

class Account {
  constructor(
    private api: ChainWrapper,
    private indexer: Indexer,
    public wallet: IWallet
  ) {
  }

  /**
   * Retrieves HOPR balance, optionally uses the cache.
   * @returns HOPR balance
   */
  public async getBalance(useCache: boolean = false): Promise<Balance> {
    return getBalance(this.api.getBalance, this.getAddress(), useCache)
  }

  /**
   * Retrieves ETH balance, optionally uses the cache.
   * @returns ETH balance
   */
  public async getNativeBalance(useCache: boolean = false): Promise<NativeBalance> {
    return getNativeBalance(this.api.getNativeBalance, this.getAddress(), useCache)
  }

  async getTicketEpoch(): Promise<UINT256> {
    const state = await this.indexer.getAccount(this.getAddress())
    if (!state || !state.counter) return UINT256.fromString('0')
    return new UINT256(state.counter)
  }

  get privateKey(): Uint8Array {
    return ethers.utils.arrayify(this.wallet.privateKey)
  }

  get publicKey(): PublicKey {
    // convert to a compressed public key
    return PublicKey.fromString(ethers.utils.computePublicKey(this.wallet.publicKey, true))
  }

  getAddress(): Address {
    return Address.fromString(this.wallet.address)
  }
}


/**
 * Retrieves HOPR balance, optionally uses the cache.
 * TODO: use indexer to track HOPR balance
 * @returns HOPR balance
 */
export const getBalance = async (
  getBalance: (account: Address) => Promise<Balance>,
  account: Address,
  useCache: boolean = false
): Promise<Balance> => {
  if (useCache) {
    const cached = cache.get('balance')
    const notExpired = cached && !isExpired(cached.updatedAt, new Date().getTime(), PROVIDER_CACHE_TTL)
    if (notExpired) return new Balance(new BN(cached.value))
  }

  const value = await getBalance(account)
  cache.set('balance', { value: value.toBN().toString(), updatedAt: new Date().getTime() })

  return value
}

/**
 * Retrieves ETH balance, optionally uses the cache.
 * @returns ETH balance
 */
export const getNativeBalance = async (
  getNativeBalance: (account: Address) => Promise<NativeBalance>,
  account: Address,
  useCache: boolean = false
): Promise<NativeBalance> => {
  if (useCache) {
    const cached = cache.get('nativeBalance')
    const notExpired = cached && !isExpired(cached.updatedAt, new Date().getTime(), PROVIDER_CACHE_TTL)
    if (notExpired) return new NativeBalance(new BN(cached.value))
  }

  const value = await getNativeBalance(account)
  cache.set('nativeBalance', { value: value.toBN().toString(), updatedAt: new Date().getTime() })

  return value
}

export default Account
