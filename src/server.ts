import Hopr from "@hoprnet/hopr-core";
import type { HoprOptions } from "@hoprnet/hopr-core";
import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import PeerInfo from "peer-info";
import PeerId from "peer-id";
import Multiaddr from "multiaddr";
import debug from 'debug'
import { encode, decode } from 'rlp'
// @ts-ignore
import Multihash from 'multihashes'
import bs58 from 'bs58'
import { addPubKey } from '@hoprnet/hopr-core/lib/utils'
import { getBootstrapAddresses } from "@hoprnet/hopr-utils"
import { commands } from '@hoprnet/hopr-chat'
import {LogStream, Socket} from './logs'
import { setupAdminServer, periodicCrawl } from './admin'
import chalk from 'chalk'
import * as yargs from 'yargs';
import { startServer } from '@hoprnet/hopr-server'

// @ts-ignore
chalk.level = 0 // We need bare strings
let debugLog = debug('hoprd')

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

const argv = (
  yargs.option('admin', {
    boolean: true,
    describe: 'Run an admin interface on localhost:3000',
    default: false
  })
  .option('grpc', {
    boolean: true,
    describe: 'Run a gRPC interface',
    default: false
  }).argv
)


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


async function main() {
  let node: Hopr<HoprCoreConnector>;
  let addr: Multiaddr;
  let logs = new LogStream()

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

  node = await Hopr.create(options);
  logs.log('Created HOPR Node')

  node.on("peer:connect", (peer: PeerInfo) => {
    logs.log(`Incoming connection from ${peer.id.toB58String()}.`);
  });

  process.once("exit", async () => {
    await node.down();
    logs.log('Process exiting')
    return;
  });

  if (argv.admin) {
    setupAdminServer(logs, node);
  }

  if (argv.grpc) {
    // Start HOPR server
    startServer(node)
  }
}
main();
