import Hopr from "@hoprnet/hopr-core";
import type { HoprOptions } from "@hoprnet/hopr-core";
import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import PeerInfo from "peer-info";
import PeerId from "peer-id";
import Multiaddr from "multiaddr";
import debug from 'debug'
import express from 'express'
import fs from 'fs'
import path from 'path'
import ws from 'ws'
import http from 'http'
import { encode, decode } from 'rlp'
// @ts-ignore
import Multihash from 'multihashes'
import bs58 from 'bs58'
import { addPubKey } from '@hoprnet/hopr-core/lib/utils'
import { getBootstrapAddresses } from "@hoprnet/hopr-utils"
import { commands } from '@hoprnet/hopr-chat'
import {LogStream, Socket} from './logs'

import chalk from 'chalk'

// @ts-ignore
chalk.level = 0 // We need bare strings

const CRAWL_TIMEOUT = 100_000 // ~15 mins

let NODE: Hopr<HoprCoreConnector>;
let debugLog = debug('hopr-admin')

/**
 * TEMPORARY HACK - copy pasted from
 * https://github.com/hoprnet/hopr-chat/blob/master/utils/checkPeerId.ts
 *
 *
 * Takes the string representation of a peerId and checks whether it is a valid
 * peerId, i. e. it is a valid base58 encoding.
 * It then generates a PeerId instance and returns it.
 *
 * @param query query that contains the peerId
 */
export async function checkPeerIdInput(query: string): Promise<PeerId> {
  let peerId: PeerId

  try {
    // Throws an error if the Id is invalid
    Multihash.decode(bs58.decode(query))

    peerId = await addPubKey(PeerId.createFromB58String(query))
  } catch (err) {
    throw Error(`Invalid peerId. ${err.message}`)
  }

  return peerId
}


// DEFAULT VALUES FOR NOW
const network = process.env.HOPR_NETWORK || "ETHEREUM";
const provider =
  process.env.HOPR_ETHEREUM_PROVIDER ||
  "wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36";
const host = process.env.HOPR_HOST || "0.0.0.0:9091"; // Default IPv4

// TODO this should probably be shared between chat and this, and live in a
// utils module.
function parseHosts(): HoprOptions['hosts'] {
  const hosts: HoprOptions['hosts'] = {}
  if (host !== undefined) {
    const str = host.replace(/\/\/.+/, '').trim()
    const params = str.match(/([0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3})\:([0-9]{1,6})/)
    if (params == null || params.length != 3) {
      throw Error(`Invalid IPv4 host. Got ${str}`)
    }

    hosts.ip4 = {
      ip: params[1],
      port: parseInt(params[2]),
    }
  }
  return hosts
}

function setupAdminServer(logs: LogStream, node: Hopr<HoprCoreConnector>){
  let cmds = new commands.Commands(node)
  var app = express()
  app.get('/', function(req, res){
    res.set('Content-Type', 'text/html')
    res.send(fs.readFileSync(path.resolve('./src/admin.html')))
  })

  const server = http.createServer(app);

  const wsServer = new ws.Server({ server: server });
  wsServer.on('connection', socket => {
    socket.on('message', message => {
      debugLog("Message from client", message)
      logs.logFullLine(`admin > ${message}`)
      cmds.execute(message.toString()).then( (resp) => {
        if (resp) {
          logs.logFullLine(resp)
        }
      })
      // TODO
    });
    socket.on('error', err => {
      debugLog('Error', err)
      logs.log('Websocket error', err.toString())
    })
    logs.subscribe(socket)
  });

  const port = process.env.HOPR_ADMIN_PORT || 3000
  server.listen(port)
  logs.log('Admin server listening on port '+ port)
}

async function main() {
  let addr: Multiaddr;
  let logs = new LogStream()

  let options: HoprOptions = {
    debug: Boolean(process.env.DEBUG),
    network,
    bootstrapServers: [... (await getBootstrapAddresses()).values()],
    provider,
    hosts: parseHosts(),
    output: logMessageToNode,
    password: process.env.HOPR_PASSWORD || 'open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0' // TODO!!!
  };

  logs.log('Creating HOPR Node')
  logs.log('- network : ' + network);
  logs.log('- bootstrapServers : ' + Array.from(options.bootstrapServers || []).map(x => x.id.toB58String()).join(','));

  NODE = await Hopr.create(options);
  logs.log('Created HOPR Node')


  setupAdminServer(logs, NODE);

  
  function logMessageToNode(msg: Uint8Array){
    logs.log("#### NODE RECEIVED MESSAGE ####")
    try {
      let [decoded, time] = decode(msg) as [Buffer, Buffer]
      logs.log("Message:", decoded.toString())
      logs.log("Latency:", Date.now() - parseInt(time.toString('hex'), 16) + 'ms')
    } catch (err) {
      logs.log("Could not decode message", err)
      logs.log(msg.toString())
    }
  }


  NODE.on("peer:connect", (peer: PeerInfo) => {
    logs.log(`Incoming connection from ${peer.id.toB58String()}.`);
  });

  process.once("exit", async () => {
    await NODE.down();
    return;
  });


  async function connectionReport(){
    logs.log(`Node is connected at ${NODE.peerInfo.id.toB58String()}`)
    logs.log(`Connected to: ${NODE.peerStore.peers.size} peers`)
    setTimeout(connectionReport, 10_000);
  }
  connectionReport()

  async function periodicCrawl(){
    try {
      await NODE.network.crawler.crawl()
      logs.log('Crawled network')
    } catch (err) {
     logs.log("Failed to crawl")
     logs.log(err)
    }
    setTimeout(periodicCrawl, CRAWL_TIMEOUT)
  }
  periodicCrawl()

  async function reportMemoryUsage(){
    const used = process.memoryUsage();
    const usage = process.resourceUsage();
    logs.log(
    `Process stats: mem ${used.rss / 1024}k (max: ${usage.maxRSS / 1024}k) ` +
    `cputime: ${usage.userCPUTime}`)
    setTimeout(reportMemoryUsage, 10_000);
  }
  reportMemoryUsage()
}

main();
