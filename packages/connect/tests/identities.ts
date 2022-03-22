import type PeerId from 'peer-id'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'

const Alice = privKeyToPeerId('0xd12c951563ee7e322562b7ce7a31c37cc6c10d9b86f834ed30f7c4ab42ae8de0')

const Bob = privKeyToPeerId('0x1a7a8c37e30c97ebf532042bdc37fe724a3950b0cd7ea5a57c9f3e30c53c44a3')

const Charly = privKeyToPeerId('0x48e7824a5816d61fb06c17adfd2458415e211a4851c74351ab49bad060a7a851')

const Dave = privKeyToPeerId('3b97195b9cde8ad2d7bdd6b5d176912e48b9c1e98f7ad269cfcd73fb5dd68d44')

const Ed = privKeyToPeerId('0xf10a7b8d92619663e75651db0f12ec83370e30edc72c12d2654e03045e1b398c')

export function peerIdForIdentity(identityName: string): PeerId {
  switch (identityName) {
    case 'alice':
      return Alice
    case 'bob':
      return Bob
    case 'charly':
      return Charly
    case 'dave':
      return Dave
    case 'ed':
      return Ed
    default:
      throw new Error(`unknown identity ${identityName}`)
  }
}

export async function identityFromPeerId(peerIdToCheck: PeerId): Promise<string> {
  for (const identityName of ['alice', 'bob', 'charly', 'dave', 'ed']) {
    const peerId = peerIdForIdentity(identityName)
    if (peerId.toB58String() === peerIdToCheck.toB58String()) {
      return identityName
    }
  }

  console.log(`can't find identity for peerId ${peerIdToCheck}`)
  return 'unknown'
}
