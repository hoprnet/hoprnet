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

let NODE: Hopr<HoprCoreConnector>;
let debugLog = debug('hopr-admin')

type Socket = ws

class LogStream {
  private messages: string[] = []
  private connections: Socket[] = []

  constructor(){
  }

  subscribe(sock: Socket){
    this.connections.push(sock);
    sock.send(this.messages.join('\n'))
  }

  log(...args: string[]){
    // @ts-ignore
    debugLog(...args) 

    this.messages.push(args.join(' '))
    if (this.messages.length > 100){ // Avoid memory leak
      this.messages.splice(0, this.messages.length - 100); // delete elements from start
    }


    this.connections.forEach((conn: Socket, i: number) => {
      if (conn.readyState == ws.OPEN) {
        conn.send(args.join(' '))
      } else {
        // Handle bad connections:
        if (conn.readyState !== ws.CONNECTING) {
          // Only other possible states are closing or closed
          this.connections.splice(i, 1)
        }

      }
    })
  }
}



// DEFAULT VALUES FOR NOW
const network = process.env.HOPR_NETWORK || "ETHEREUM";
const provider =
  process.env.HOPR_ETHEREUM_PROVIDER ||
  "wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36";
const bootstrapAddresses =
  (process.env.HOPR_BOOTSTRAP_SERVERS ||
  ["/ip4/34.65.82.167/tcp/9091/p2p/16Uiu2HAm6VH37RG1R4P8hGV1Px7MneMtNc6PNPewNxCsj1HsDLXW",
    "/ip4/34.65.111.179/tcp/9091/p2p/16Uiu2HAmPyq9Gw93VWdS3pgmyAWg2UNnrgZoYKPDUMbKDsWhzuvb"]);
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
  let addr: Multiaddr;
  let bootstrapServerMap = new Map<string, PeerInfo>();
  let logs = new LogStream()

  if (bootstrapAddresses.length == 0) {
    throw new Error(
      "Invalid bootstrap servers. Cannot start HOPR without a bootstrap node"
    );
  }

  for (let i = 0; i < bootstrapAddresses.length; i++) {
    addr = Multiaddr(bootstrapAddresses[i].trim());
    let peerInfo = bootstrapServerMap.get(addr.getPeerId());
    if (peerInfo == null) {
      peerInfo = await PeerInfo.create(
        PeerId.createFromB58String(addr.getPeerId())
      );
    }

    peerInfo.multiaddrs.add(addr);
    bootstrapServerMap.set(addr.getPeerId(), peerInfo);
  }

  let options: HoprOptions = {
    debug: Boolean(process.env.DEBUG),
    network,
    bootstrapServers: [...bootstrapServerMap.values()],
    provider,
    hosts: parseHosts(),
    password: process.env.HOPR_PASSWORD || 'open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0' // TODO!!!
  };

  debugLog(options)
  logs.log('Creating HOPR Node')
  logs.log('- network : ' + network);
  logs.log('- bootstrapServers : ' + Array.from(bootstrapServerMap.values()).join(', '));

  NODE = await Hopr.create(options);
  logs.log('Created HOPR Node')

  NODE.on("peer:connect", (peer: PeerInfo) => {
    logs.log(`Incoming connection from ${peer.id.toB58String()}.`);
  });

  process.once("exit", async () => {
    await NODE.down();
    return;
  });


  async function periodicCrawl(){
    try {
      await NODE.network.crawler.crawl()
      logs.log('Crawled network')
    } catch (err) {
     logs.log("Failed to crawl")
     logs.log(err)
    }
    setTimeout(periodicCrawl, 10_000)
  }

  periodicCrawl()

  // Static file server
  var app = express()
  app.get('/', function(req, res){
    res.set('Content-Type', 'text/html')
    res.send(fs.readFileSync(path.resolve('./src/admin.html')))
  })

  const server = http.createServer(app);

  const wsServer = new ws.Server({ server: server });
  wsServer.on('connection', socket => {
    socket.on('message', message => debugLog("Message from client", message));
    logs.subscribe(socket)
  });

  server.listen(process.env.HOPR_ADMIN_PORT || 3000)
}

main();
