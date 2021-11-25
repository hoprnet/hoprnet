import { Multiaddr } from 'multiaddr'
import { Address, privKeyToPeerId } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'

export const NAMESPACE = 'hopr:mocks'

export const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'
export const mockPeerId = privKeyToPeerId(privateKey)
export const sampleAddress = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
export const samplePeerId = PeerId.createFromB58String('16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7')
export const sampleMultiaddrs = new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${samplePeerId.toB58String()}`)

