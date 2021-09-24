import { findPath } from '@hoprnet/hopr-core'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { PublicKey, ChannelEntry } from '@hoprnet/hopr-utils'
import type { State, ChannelData, PersistedState } from './state'
import { CT_PATH_RANDOMNESS, CT_INTERMEDIATE_HOPS } from './constants'
import debug from 'debug'

const log = debug('cover-traffic')

export const addBN = (a: BN, b: BN): BN => a.add(b)
export const sqrtBN = (a: BN): BN => new BN(new BigNumber(a.toString()).squareRoot().integerValue().toFixed(), 10)

/**
 * Get channels opened from a node with a given public key in the state.
 * @param p Public key of the `source` node
 * @returns a list of channel entries where the `source` is the given public key
 */
export const findChannelsFrom = (p: PublicKey, state: State): ChannelEntry[] =>
  Object.values(state.channels)
    .map((c) => c.channel)
    .filter((c: ChannelEntry) => c.source.eq(p))

/**
 * Get the total outgoing channel balance of a node, given the network state.
 * totalChannelBalance(n) = balance of each channel from node n
 * @param p Public key of the node
 * @param state State of the network
 * @returns Total channel balance in big number
 */
export const totalChannelBalanceFor = (p: PublicKey, state: State): BN =>
  findChannelsFrom(p, state)
    .map((c) => c.balance.toBN())
    .reduce(addBN, new BN('0'))

/**
 * Get the importance score of a node, given the network state.
 * Sum of the square root of all the outgoing channels
 * importance(node) = sum(squareRoot((balance(node) * stake(node) * totalStake(node)))
 * where stake(n) = unreleasedTokens(n) + totalChannelBalance(n)
 * FIXME: Current version does not contain unreleased token balance.
 * @param p Public key of the node
 * @param state State of the network
 * @returns Total channel balance in big number
 */
export const importance = (p: PublicKey, state: State): BN =>
  findChannelsFrom(p, state)
    .map((c: ChannelEntry) =>
      sqrtBN(totalChannelBalanceFor(p, state).mul(c.balance.toBN()).mul(totalChannelBalanceFor(c.destination, state)))
    )
    .reduce(addBN, new BN('0'))

/**
 * Return the randomnized importance score of a node, given the network state.
 * @param p Public key of the node
 * @param state State of the network
 * @returns the randmonized importance score
 */
export const randomWeightedImportance = (p: PublicKey, state: State): BN => {
  const randomComponent = 1 + Math.random() * CT_PATH_RANDOMNESS
  return importance(p, state).muln(randomComponent)
}

/**
 * Find the channel entry that is between the provided source and destination,
 * given the network state.
 * @param src Public key of the `source` of the channel
 * @param dest Public key of the `destination` of the channel
 * @returns ChannelEntry between `source` and `destination`, undefined otherwise.
 */
export const findChannel = (src: PublicKey, dest: PublicKey, state: State): ChannelEntry =>
  Object.values(state.channels)
    .map((c: ChannelData): ChannelEntry => c.channel)
    .find((c: ChannelEntry) => c.source.eq(src) && c.destination.eq(dest))

/*
 * Find the timestamp at which a CT channel is opened.
 * @param dest Public key of the `destination` of the channel
 * @param state State of the network
 */
export const findCtChannelOpenTime = (dest: PublicKey, state: State): number => {
  const ctChannel = state.ctChannels.find((ctChannel) => ctChannel.destination === dest)
  return !ctChannel ? Date.now() : ctChannel.openFrom ?? Date.now()
}

/**
 *
 * @param startNode Public key of the first-hop node
 * @param selfPub Public key of the cover-traffic node, which is also the recipient of the message.
 * @param sendMessage Method to send messages
 * @param data Persisted state of the network.
 * @returns false if no path is found.
 */
export const sendCTMessage = async (
  startNode: PublicKey,
  selfPub: PublicKey,
  sendMessage: (path: PublicKey[]) => Promise<void>,
  data: PersistedState
): Promise<boolean> => {
  // get the randomized weighted importance score of the destination of a given channel.
  const weight = async (edge: ChannelEntry): Promise<BN> => randomWeightedImportance(edge.destination, data.get())
  let path: PublicKey[]
  try {
    path = await findPath(
      startNode,
      selfPub,
      CT_INTERMEDIATE_HOPS - 1, // As us to start is first intermediate
      (_p: PublicKey): number => 1, // TODO: network quality?
      (p: PublicKey) => Promise.resolve(data.findChannelsFrom(p)),
      weight
    )

    path.forEach((p) => data.incrementForwards(p))
    log('SEND ' + path.map((pub) => pub.toB58String()).join(','))
  } catch (e) {
    // could not find path
    log(`Could not find path: ${startNode.toB58String()} -> ${selfPub.toPeerId()} (${e})`)
    return false
  }
  try {
    data.incrementSent(startNode)
    await sendMessage(path)
    return true
  } catch (e) {
    //console.log(e)
    log('error sending to' + startNode.toPeerId().toB58String())
    return false
  }
}
