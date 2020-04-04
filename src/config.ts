import { Networks } from './tsc/types'

export const DEFAULT_URI = 'ws://127.0.0.1:9545/'

export const TOKEN_ADDRESSES: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: '0x25f4408b0F75D1347335fc625E7446F2CdEcD503',
  goerli: undefined,
  kovan: undefined,
  private: '0x0f5Ea0A652E851678Ebf77B69484bFcD31F9459B'
}

export const CHANNELS_ADDRESSES: { [key in Networks]: string } = {
  mainnet: undefined,
  morden: undefined,
  ropsten: undefined,
  rinkeby: '0x077209b19F4Db071254C468E42784588003be34C',
  goerli: undefined,
  kovan: undefined,
  private: '0xEC8bE1A5630364292E56D01129E8ee8A9578d7D8'
}

export const FUND_ACCOUNT_PRIVATE_KEY = '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501200'

/**
 * These private keys serve as demo identities for local testnets.
 * Private keys are the same as in `hopr-ethereum` repository.
 *
 * @notice Do NOT use them to store any meaningful assets!
 */
export const DEMO_ACCOUNTS = [
  FUND_ACCOUNT_PRIVATE_KEY,
  // Alice
  '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501201',
  // Bob
  '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501202',
  // Chris
  '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501203',
  // Dave
  '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501204',
  // Ed
  '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501205',
  // Fred
  '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501206',
  // George
  '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501207',
  // Henry
  '0x2bdd21761a483f71054e14f5b827213567971c676928d9a1808cbfa4b7501208'
]
