import { findPath } from '@hoprnet/hopr-core'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { PublicKey, ChannelEntry } from '@hoprnet/hopr-utils'
import type { State, ChannelData, PersistedState } from './state'
import { CT_PATH_RANDOMNESS, CT_INTERMEDIATE_HOPS } from './constants'

export const addBN = (a: BN, b: BN): BN => a.add(b)
export const sqrtBN = (a: BN): BN => new BN(new BigNumber(a.toString()).squareRoot().integerValue().toFixed(), 10)

export const findChannelsFrom = (p: PublicKey, state: State): ChannelEntry[] =>
  Object.values(state.channels)
    .map((c) => c.channel)
    .filter((c: ChannelEntry) => c.source.eq(p))

export const totalChannelBalanceFor = (p: PublicKey, state: State): BN =>
  findChannelsFrom(p, state)
    .map((c) => c.balance.toBN())
    .reduce(addBN, new BN('0'))

export const importance = (p: PublicKey, state: State): BN =>
  findChannelsFrom(p, state)
    .map((c: ChannelEntry) =>
      sqrtBN(totalChannelBalanceFor(p, state).mul(c.balance.toBN()).mul(totalChannelBalanceFor(c.destination, state)))
    )
    .reduce(addBN, new BN('0'))

export const randomWeightedImportance = (p: PublicKey, state: State): BN => {
  const randomComponent = 1 + Math.random() * CT_PATH_RANDOMNESS
  return importance(p, state).muln(randomComponent)
}

export const findChannel = (src: PublicKey, dest: PublicKey, state: State): ChannelEntry =>
  Object.values(state.channels)
    .map((c: ChannelData): ChannelEntry => c.channel)
    .find((c: ChannelEntry) => c.source.eq(src) && c.destination.eq(dest))

export const sendCTMessage = async (
  startNode: PublicKey,
  selfPub: PublicKey,
  sendMessage: (path: PublicKey[]) => Promise<void>,
  data: PersistedState
): Promise<boolean> => {
  const weight = async (edge: ChannelEntry): Promise<BN> => randomWeightedImportance(edge.destination, data.get())
  let path: PublicKey[]
  try {
    path = await findPath(
      startNode,
      selfPub,
      CT_INTERMEDIATE_HOPS - 1, // As us to start is first intermediate
      (_p: PublicKey): number => 1, // TODO network quality?
      (p: PublicKey) => Promise.resolve(data.findChannelsFrom(p)),
      weight
    )

    path.forEach((p) => data.incrementForwards(p))
    path.push(selfPub) // destination is always self.
    data.log('SEND ' + path.map((pub) => pub.toB58String()).join(','))
  } catch (e) {
    // could not find path
    data.log(`Could not find path: ${startNode.toB58String()} -> ${selfPub.toPeerId()} (${e})`)
    return false
  }
  try {
    data.incrementSent(startNode)
    await sendMessage(path)
    return true
  } catch (e) {
    //console.log(e)
    data.log('error sending to' + startNode.toPeerId().toB58String())
    return false
  }
}
