/// <reference path="../src/@types/stream-to-it.ts" />
/// <reference path="../src/@types/it-handshake.ts" />

import libp2p from 'libp2p'
import type { Connection, HandlerProps } from 'libp2p'
import { durations } from '@hoprnet/hopr-utils'
import fs from 'fs'

import { NOISE } from 'libp2p-noise'

const MPLEX = require('libp2p-mplex')

import { HoprConnect } from '../src'
import type { HoprConnectOptions } from '../src'
import { Multiaddr } from 'multiaddr'
import pipe from 'it-pipe'
import yargs from 'yargs/yargs'
import { peerIdForIdentity, identityFromPeerId } from './identities'
import type PeerId from 'peer-id'
import type { WriteStream } from 'fs'
import type { PeerStoreType, Stream } from '../src/types'

const TEST_PROTOCOL = '/hopr-connect/test/0.0.1'

function encodeMsg(msg: string): Uint8Array {
  return new TextEncoder().encode(msg)
}

function decodeMsg(encodedMsg: Uint8Array): string {
  return new TextDecoder().decode(encodedMsg)
}

function createEchoReplier(remoteIdentityname: string, pipeFileStream?: WriteStream) {
  return (source: Stream['source']) => {
    return (async function* () {
      for await (const encodedMsg of source) {
        const decodedMsg = decodeMsg(encodedMsg.slice())
        const replyMsg = `echo: ${decodedMsg}`

        console.log(`received message '${decodedMsg}' from ${remoteIdentityname}`)
        console.log(`replied with ${replyMsg}`)

        if (pipeFileStream) {
          pipeFileStream.write(`<${remoteIdentityname}: ${decodedMsg}\n`)
          pipeFileStream.write(`>${remoteIdentityname}: ${replyMsg}\n`)
        }
        yield encodeMsg(replyMsg)
      }
    })()
  }
}

function createDeadEnd(remoteIdentityname: string, pipeFileStream?: WriteStream) {
  return async (source: Stream['source']) => {
    for await (const encodedMsg of source) {
      const decodedMsg = decodeMsg(encodedMsg.slice())
      console.log(`received message '${decodedMsg}' from ${remoteIdentityname}`)
      console.log(`didn't reply`)

      if (pipeFileStream) {
        pipeFileStream.write(`<${remoteIdentityname}: ${decodedMsg}\n`)
      }
    }
  }
}

async function startNode({
  peerId,
  port,
  bootstrapAddress,
  noDirectConnections,
  noWebRTCUpgrade,
  pipeFileStream,
  maxRelayedConnections,
  relayFreeTimeout
}: {
  peerId: PeerId
  port: number
  bootstrapAddress?: PeerStoreType
  noDirectConnections: boolean
  noWebRTCUpgrade: boolean
  pipeFileStream?: WriteStream
  maxRelayedConnections?: number
  relayFreeTimeout?: number
}) {
  console.log(`starting node, bootstrap address ${bootstrapAddress}`)
  const connectOpts: HoprConnectOptions = {
    initialNodes: bootstrapAddress ? [bootstrapAddress] : [],
    __noDirectConnections: noDirectConnections,
    __noWebRTCUpgrade: noWebRTCUpgrade,
    maxRelayedConnections,
    __relayFreeTimeout: relayFreeTimeout
  }

  const node = await libp2p.create({
    peerId,
    addresses: {
      listen: [`/ip4/0.0.0.0/tcp/${port}/p2p/${peerId.toB58String()}`]
    },
    modules: {
      transport: [HoprConnect as any],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE]
    },
    config: {
      transport: {
        HoprConnect: connectOpts
      },
      peerDiscovery: {
        autoDial: false
      }
    },
    dialer: {
      // Temporary fix
      addressSorter: (ma: any) => ma
    }
  })

  async function identityNameForConnection(connection?: Connection): Promise<string> {
    if (!connection) {
      return 'unknown'
    }
    return identityFromPeerId(connection.remotePeer)
  }

  node.handle(TEST_PROTOCOL, async (conn: HandlerProps) => {
    pipe(
      conn.stream.source,
      createEchoReplier(await identityNameForConnection(conn.connection), pipeFileStream),
      conn.stream.sink
    )
  })

  await node.start()
  console.log(`node started`)
  return node
}

type CmdDef =
  | {
      cmd: 'wait'
      waitForSecs: number
    }
  | {
      cmd: 'dial'
      targetIdentityName: string
      targetPort: number
    }
  | {
      cmd: 'msg'
      msg: string
      targetIdentityName: string
      relayIdentityName: string
    }
  | {
      cmd: 'hangup'
      targetIdentityName: string
    }

async function executeCommands({
  node,
  cmds,
  pipeFileStream
}: {
  node: libp2p
  cmds: CmdDef[]
  pipeFileStream?: WriteStream
}) {
  for (const cmdDef of cmds) {
    switch (cmdDef.cmd) {
      case 'wait': {
        console.log(`waiting ${cmdDef.waitForSecs} secs`)
        await new Promise((resolve) => setTimeout(resolve, durations.seconds(cmdDef.waitForSecs)))
        console.log(`finished waiting`)
        break
      }
      case 'dial': {
        const targetPeerId = await peerIdForIdentity(cmdDef.targetIdentityName)
        const targetAddress = new Multiaddr(`/ip4/127.0.0.1/tcp/${cmdDef.targetPort}/p2p/${targetPeerId.toB58String()}`)
        console.log(`dialing ${cmdDef.targetIdentityName}`)
        await node.dial(targetAddress)

        console.log(`dialed`)
        break
      }
      case 'msg': {
        const targetPeerId = await peerIdForIdentity(cmdDef.targetIdentityName)
        const relayPeerId = await peerIdForIdentity(cmdDef.relayIdentityName)

        console.log(`msg: dialing ${cmdDef.targetIdentityName} though relay ${cmdDef.relayIdentityName}`)
        const { stream } = await node
          .dialProtocol(
            new Multiaddr(`/p2p/${relayPeerId}/p2p-circuit/p2p/${targetPeerId.toB58String()}`),
            TEST_PROTOCOL
          )
          .catch((err) => {
            console.log(`dialProtocol to ${cmdDef.targetIdentityName} failed`)
            console.log(err)
            process.exit(1)
          })

        console.log(`sending msg '${cmdDef.msg}'`)

        const encodedMsg = encodeMsg(cmdDef.msg)
        if (pipeFileStream) {
          pipeFileStream.write(`>${cmdDef.targetIdentityName}: ${cmdDef.msg}\n`)
        }
        await pipe([encodedMsg], stream, createDeadEnd(cmdDef.targetIdentityName, pipeFileStream))
        console.log(`sent ok`)
        break
      }
      case 'hangup': {
        const targetPeerId = await peerIdForIdentity(cmdDef.targetIdentityName)
        console.log(`hanging up on ${cmdDef.targetIdentityName}`)
        await node.hangUp(targetPeerId)
        console.log(`hanged up`)
        break
      }
      default: {
        throw new Error(`unknown cmd: ${cmdDef}`)
      }
    }
  }
}

async function main() {
  const argv = yargs(process.argv.slice(2))
    .option('port', {
      describe: 'node port',
      type: 'number',
      demandOption: true
    })
    .option('identityName', {
      describe: 'node identity name',
      choices: ['alice', 'bob', 'charly', 'dave', 'ed'],
      demandOption: true
    })
    .option('bootstrapPort', {
      describe: 'bootstrap node port',
      type: 'number'
    })
    .option('bootstrapIdentityName', {
      describe: 'identity name of a boostrap server',
      choices: ['alice', 'bob', 'charly', 'dave', 'ed']
    })
    .option('noDirectConnections', {
      type: 'boolean',
      demandOption: true
    })
    .option('noWebRTCUpgrade', {
      type: 'boolean',
      demandOption: true
    })
    .option('command', {
      describe: 'example: --command.name dial --command.targetIdentityName charly',
      type: 'string'
    })
    .option('script', {
      type: 'string',
      demandOption: true
    })
    .option('pipeFile', {
      type: 'string'
    })
    .option('maxRelayedConnections', {
      type: 'number'
    })
    .option('relayFreeTimeout', {
      type: 'number'
    })
    .coerce({
      script: (input) => JSON.parse(input.replace(/'/g, '"'))
    })
    .parseSync()

  let bootstrapAddress: PeerStoreType | undefined

  if (argv.bootstrapPort != null && argv.bootstrapIdentityName != null) {
    const bootstrapPeerId = await peerIdForIdentity(argv.bootstrapIdentityName)
    bootstrapAddress = {
      id: bootstrapPeerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${argv.bootstrapPort}/p2p/${bootstrapPeerId.toB58String()}`)]
    }
  }
  const peerId = await peerIdForIdentity(argv.identityName)

  let pipeFileStream: WriteStream | undefined
  if (argv.pipeFile) {
    pipeFileStream = fs.createWriteStream(argv.pipeFile)
  }

  console.log(`running node ${argv.identityName} on port ${argv.port}`)
  const node = await startNode({
    peerId,
    port: argv.port,
    bootstrapAddress,
    noDirectConnections: argv.noDirectConnections,
    noWebRTCUpgrade: argv.noWebRTCUpgrade,
    pipeFileStream,
    maxRelayedConnections: argv.maxRelayedConnections,
    relayFreeTimeout: argv.relayFreeTimeout
  })

  await executeCommands({ node, cmds: argv.script, pipeFileStream })

  console.log(`all tasks executed`)
}

process.on('unhandledRejection', (error) => {
  console.log('unhandledRejection', error)
  process.exit(1)
})

main()
