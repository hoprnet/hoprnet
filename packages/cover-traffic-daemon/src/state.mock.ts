import { Balance, ChannelEntry, ChannelStatus, debug, Hash, PublicKey, stringToU8a, UINT256 } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { peerIdFromString } from '@libp2p/peer-id'
import { Multiaddr } from '@multiformats/multiaddr'
import { OpenChannels, PeerData, PersistedState, State, deserializeState, serializeState } from './state.js'

export const log = debug('hopr:cover-traffic:mock')

export const sampleData = {} as unknown as PersistedState

/**
 * Just as the PersistedState class, except that it does not
 * do any filesystem operation. Thus, the state is not persisted.
 * @dev do *not* use in production
 */
export class TestingPersistedState extends PersistedState {
  private dbString: string

  override load(): void {
    this._data = deserializeState(this.dbString)
  }

  override set(s: State): void {
    this._data = s
    this.dbString = serializeState(s)
    this.update(s)
    return
  }
}

export const mockPersistedState: PersistedState = new TestingPersistedState((state: State) => {
  log(`State update: ${Object.keys(state.nodes).length} nodes, ${Object.keys(state.channels).length} channels`)
}, './test/db.json')

export const mockState: State = {
  nodes: {},
  channels: {},
  ctChannels: [],
  block: new BN('123456'),
  messageFails: {},
  messageTotalSuccess: 0
}

export const PARTY_A = PublicKey.fromPrivKey(
  stringToU8a('0x0f1b0de97ef1e907d8152bdfdaa39b4bb5879d5d48d152a84421bd2f9ccb3877')
)

export const PARTY_B = PublicKey.fromPrivKey(
  stringToU8a('0x4c6a00ceb8e3c0c4c528839f88f2eff948dd8df37e067a8b6f222c6496bdb7b0')
)

export const mockChannelEntry = new ChannelEntry(
  PARTY_A,
  PARTY_B,
  new Balance(new BN('2')),
  new Hash(new Uint8Array({ length: Hash.SIZE })),
  new UINT256(new BN('0')),
  new UINT256(new BN('0')),
  ChannelStatus.Open,
  new UINT256(new BN('0')),
  new UINT256(new BN('0'))
)

const mockPeerId = peerIdFromString('16Uiu2HAm6fJyjpFFbFtNx2aqRakVCjodRUoagu6Pu4w1LAKL9uLy')
export const mockPublicKey = PublicKey.fromPeerId(mockPeerId)

export const mockPeerData: PeerData = {
  id: mockPeerId,
  pub: mockPublicKey,
  multiaddrs: [new Multiaddr('/ip4/127.0.0.1/udp/1234')]
}

export const mockOpenChannel: OpenChannels = {
  destination: PARTY_B,
  latestQualityOf: 1,
  openFrom: 0
}
