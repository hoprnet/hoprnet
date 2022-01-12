import { findPath } from '@hoprnet/hopr-core'
import BN from 'bn.js'
import { BigNumber } from 'bignumber.js'
import { PublicKey, ChannelEntry } from '@hoprnet/hopr-utils'
import type { State, ChannelData, PersistedState } from './state'
import { CT_PATH_RANDOMNESS, CT_INTERMEDIATE_HOPS } from './constants'
import { debug } from '@hoprnet/hopr-utils'

const log = debug('hopr:cover-traffic')

/**
 * Interval of the unreleased token schedule. Unrecordeed intervals has 0 unreleased token allocation.
 * @param lowerBlock The lower bound of the time interval during which the unreleased token amount is defined
 * @param unreleased The unreleased token amount in 18 decimals
 */
export type UnreleasedSchedule = {
  lowerBlock: number
  upperBlock: number
  unreleased: string
}

export type UnreleasedTokens = {
  // hopr id to node Ethereum addresses
  link: Record<string, string[]>
  // Node Ethereum address to the array of unreleased token schedule
  allocation: Record<string, UnreleasedSchedule[]>
}

const unreleasedTokens: UnreleasedTokens = require('../unreleasedTokens.json')

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
 * Get the stake of a node which consists of the total channel balance
 * and the unreleased token amount in big number
 *
 * stake(n) = unreleasedTokens(n) + totalChannelBalance(n)
 *
 * @param p Public key of the node
 * @param state State of the network
 * @returns Stake of a node in big number
 */
export const stakeFor = (p: PublicKey, state: State): BN => {
  const linkedAccountsIndex = Object.keys(unreleasedTokens.link).findIndex(
    (id) => id.toLowerCase() == p.toB58String().toLowerCase()
  )

  if (linkedAccountsIndex < 0) {
    return totalChannelBalanceFor(p, state)
  }

  const currentBlockNumber = state.block.toNumber()

  return Object.values(unreleasedTokens.link)
    [linkedAccountsIndex].map((nodeAddress) => {
      const scheduleIndex = unreleasedTokens.allocation[nodeAddress].findIndex(
        (schedule) => schedule.lowerBlock <= currentBlockNumber && currentBlockNumber < schedule.upperBlock
      )
      return scheduleIndex < 0
        ? new BN('0')
        : new BN(unreleasedTokens.allocation[nodeAddress][scheduleIndex].unreleased)
    })
    .reduce(addBN, new BN('0'))
    .add(totalChannelBalanceFor(p, state))
}

/**
 * Get the importance score of a node, given the network state.
 * Sum of the square root of all the outgoing channels
 * importance(node) = sum(squareRoot((balance(node) * stake(node) * totalStake(node)))
 * where stake(n) = unreleasedTokens(n) + totalChannelBalance(n)
 * @param p Public key of the node
 * @param state State of the network
 * @returns Total channel balance in big number
 */
export const importance = (p: PublicKey, state: State): BN =>
  findChannelsFrom(p, state)
    .map((c: ChannelEntry) => sqrtBN(stakeFor(p, state).mul(c.balance.toBN()).mul(stakeFor(c.destination, state))))
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
  sendMessage: (message: Uint8Array, path: PublicKey[]) => Promise<void>,
  data: PersistedState
): Promise<boolean> => {
  // get the randomized weighted importance score of the destination of a given channel.
  const weight = async (edge: ChannelEntry): Promise<BN> => randomWeightedImportance(edge.destination, data.get())
  let path: PublicKey[]

  // build CT message,
  const counter = data.messageTotalSuccess()
  const message = new TextEncoder().encode(`CT_${counter.toString}`)

  try {
    path = await findPath(
      startNode,
      selfPub,
      CT_INTERMEDIATE_HOPS - 1, // As us to start is first intermediate
      (_p: PublicKey): number => 1, // TODO: network quality?
      (p: PublicKey) => Promise.resolve(data.findChannelsFrom(p)),
      weight
    )

    // update counters in the state
    path.forEach((p, i, a) => {
      // increase counter for non-1st hop nodes
      if (i < CT_INTERMEDIATE_HOPS - 2) {
        data.incrementForwards(p, a[i + 1])
      } else {
        data.incrementForwards(p, selfPub)
      }
    })
    data.incrementSent(startNode, path[0]) // increase counter for 1st hop node

    // build the complete path
    path.unshift(startNode) // Path doesn't normally include this

    log(`SEND ${path.map((pub) => pub.toB58String()).join(',')}`)
  } catch (e) {
    return false
  }
  try {
    await sendMessage(message, path)
    log(`success sending ${path.map((pub) => pub.toB58String()).join(',')} message ${message}`)
    return true
  } catch (e) {
    log(`error ${e} sending to ${startNode.toB58String()}`)
    return false
  }
}
