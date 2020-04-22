import { NODE_SEEDS } from '@hoprnet/hopr-demo-seeds'
import { Networks } from './tsc/types'

export const DEFAULT_URI = 'ws://127.0.0.1:9545/'

export const TOKEN_ADDRESSES: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: '0x25f4408b0F75D1347335fc625E7446F2CdEcD503',
  goerli: undefined,
  kovan: '0x591aE064387AB09805D9fF9206F0A90DB8F4C9B2',
  private: '0x04B9036C7Af0d40127310D04E1A34267Cebc7B36'
}

export const CHANNELS_ADDRESSES: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: '0x077209b19F4Db071254C468E42784588003be34C',
  goerli: undefined,
  kovan: '0x506De99826736032cF760586ECce0bae1369155b',
  private: '0x25Dbc956D8b6a5cB2089A62c0F0F839C2eCf2bEb'
}

export const FUND_ACCOUNT_PRIVATE_KEY = NODE_SEEDS[0]
export const DEMO_ACCOUNTS = NODE_SEEDS
