import { splitPubKey } from '../utils'

// accountA == partyA
export const ACCOUNT_A_PUBKEY = splitPubKey(
  '0x04b1a05c601afd23f4bb9a066cd1900dedbbd61314faf467394e25ec479050f5268b70662eb562be188f2ada7648054e28fa6ee5b774e71d5b5643c07d3d120c25'
)
export const ACCOUNT_A_ADDRESS = '0x4f022DBa1DA28E1cde77a320832828896c773388'

// accountB == partyB
export const ACCOUNT_B_PUBKEY = splitPubKey(
  '0x04f74c37e5ea4ff89bde83c8e3910fbbb58ba094c6d87346fb93fcccf008e5f3985cb3203fcd808ca41d82dd6f366f5dde53dfeabbdf3ed20f80f59fc8b05aac5e'
)
export const ACCOUNT_B_ADDRESS = '0x86baf62545b0293eCa84a29F1F578FEb5Df1F6E2'

// channel id of accountA and accountB
export const ACCOUNT_AB_CHANNEL_ID = '0xa5cddce12b5dafc886e3d28f93b8659d2a9afbc8cb704a3a0d17d967771dc807'

export const SECRET_PRE_IMAGE = '0xc89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6'
export const SECRET = '0x4aeff0db81e3146828378be230d377356e57b6d599286b4b517dbf8941b3e1b2'
