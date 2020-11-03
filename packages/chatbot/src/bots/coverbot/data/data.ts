import debug from 'debug'
import { Coverbot } from '..'
import { Table } from 'console-table-printer'
import { databaseTextRef, stateDbRef, scoreDbRef } from '../utils/constants'
import {
  COVERBOT_DEBUG_MODE,
  COVERBOT_CHAIN_PROVIDER,
  COVERBOT_VERIFICATION_CYCLE_IN_MS,
  COVERBOT_XDAI_THRESHOLD,
  HOPR_ENVIRONMENT,
} from '../../../utils/env'
import { Utils } from '@hoprnet/hopr-core-ethereum'
import { HoprNode } from '../types/coverbot'
import addresses from '@hoprnet/hopr-ethereum/lib/chain/addresses'
import type { Network } from '@hoprnet/hopr-ethereum/lib/utils/networks'
import Web3 from 'web3'
import { convertPubKeyFromB58String, u8aToHex } from '@hoprnet/hopr-utils'

const log = debug('hopr-chatbot:data')
const error = debug('hopr-chatbot:data:error')
const { fromWei } = Web3.utils

export async function _getEthereumAddressFromHOPRAddress(hoprAddress: string): Promise<string> {
  const pubkey = await convertPubKeyFromB58String(hoprAddress)
  const ethereumAddress = u8aToHex(await Utils.pubKeyToAccountId(pubkey.marshal()))
  return ethereumAddress
}

export async function _getEthereumAddressScore(ethereumAddress: string): Promise<number> {
  return new Promise((resolve, reject) => {
    scoreDbRef.child(ethereumAddress).once('value', (snapshot, error) => {
      if (error) return reject(error)
      return resolve(snapshot.val() || 0)
    })
  })
}

export async function _setEthereumAddressScore(ethereumAddress: string, score: number): Promise<void> {
  return new Promise((resolve, reject) => {
    scoreDbRef.child(ethereumAddress).setWithPriority(score, -score, (error) => {
      if (error) return reject(error)
      return resolve()
    })
  })
}

export async function loadData(this: Coverbot): Promise<void> {
  log(`- loadData | Loading data from Database (${databaseTextRef})`)
  return new Promise((resolve, reject) => {
    this.database
      .getTable(HOPR_ENVIRONMENT, 'state')
      .then((state) => {
        if (!state) {
          log(`- loadData | Database (${databaseTextRef}) hasn’t been created`)
          return resolve()
        }
        const { env, connected = [], ...substate } = state
        log(`- loadData | Env: ${JSON.stringify(env)} obtained from database`)
        log(`- loadData | Loaded ${connected.length} nodes from our Database (${databaseTextRef})`)

        this.verifiedHoprNodes =
          this.verifiedHoprNodes.values.length > 0 ? this.verifiedHoprNodes : new Map<string, HoprNode>()
        connected.forEach((n) => this.verifiedHoprNodes.set(n.id, n))
        log(`- loadData | Updated ${Array.from(this.verifiedHoprNodes.values()).length} verified nodes in memory`)

        this.loadedDb = true
        return resolve()
      })
      .catch((err) => {
        error(`- loadData | Error retrieving data`, err)
        if (err) return reject(err)
      })
  })
}

export async function dumpData(this: Coverbot) {
  log(`- dumpData | Starting dumping data in Database (${databaseTextRef})`)
  //@TODO: Ideally we move this to a more suitable place.
  if (!this.ethereumAddress) {
    this.chainId = await Utils.getChainId(this.xdaiWeb3)
    this.network = Utils.getNetworkName(this.chainId) as Network
    this.ethereumAddress = await _getEthereumAddressFromHOPRAddress(this.address)
  }

  const connectedNodes = this.node.listConnectedPeers()
  log(`- loadData | Detected ${connectedNodes} in the network w/bootstrap servers ${this.node.getBootstrapServers()}`)

  const state = {
    connectedNodes,
    env: {
      COVERBOT_CHAIN_PROVIDER,
      COVERBOT_DEBUG_MODE,
      COVERBOT_VERIFICATION_CYCLE_IN_MS,
      COVERBOT_XDAI_THRESHOLD,
    },
    hoprCoverbotAddress: await _getEthereumAddressFromHOPRAddress(this.address),
    hoprChannelContract: addresses[this.network].HoprChannels,
    address: this.address,
    balance: fromWei(await this.xdaiWeb3.eth.getBalance(this.ethereumAddress)),
    available: fromWei(await this.node.getHoprBalance()),
    locked: 0, //@TODO: Retrieve balances from open channels.
    connected: Array.from(this.verifiedHoprNodes.values()),
    refreshed: new Date().toISOString(),
  }

  return new Promise((resolve, reject) => {
    stateDbRef.set(state, (error) => {
      if (error) return reject(error)
      log(`- dumpData | Saved data in Database (${databaseTextRef})`)
      return resolve()
    })
  })
}
