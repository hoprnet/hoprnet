import { Wallet } from 'ethers'
import { Multiaddr } from '@multiformats/multiaddr'
import { PublicKey, Hash, stringToU8a } from '@hoprnet/hopr-utils'

export const ACCOUNT_A = new Wallet('0x18a664889e28a432495758f0522b53b2f04a35f810b78c6ea01db305141bcba2')
export const PARTY_A = PublicKey.deserialize(stringToU8a(ACCOUNT_A.publicKey))
export const PARTY_A_MULTIADDR = new Multiaddr(`/ip4/34.65.237.196/tcp/9091/p2p/${PARTY_A.to_peerid_str()}`)
export const ACCOUNT_B = new Wallet('0x4471496ef88d9a7d86a92b7676f3c8871a60792a37fae6fc3abc347c3aa3b16b')
export const PARTY_B = PublicKey.deserialize(stringToU8a(ACCOUNT_B.publicKey))
export const PARTY_B_MULTIADDR = new Multiaddr(`/ip4/34.65.237.197/tcp/9091/p2p/${PARTY_B.to_peerid_str()}`)
export const CHANNEL_ID = '0x6e454104cde7f1c088b14c3ead07945f6f2c1ce72fef4171a7670e528d1a043c'

export const SECRET_1 = new Hash(stringToU8a('0xb8b37f62ec82443e5b5557c5a187fe3686790620cc04c06187c48f8636caac89'))
export const SECRET_2 = new Hash(stringToU8a('0x294549f8629f0eeb2b8e01aca491f701f5386a9662403b485c4efe7d447dfba3'))
