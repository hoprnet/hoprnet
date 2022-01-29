import { default as libp2p, type Connection, type HandlerProps } from 'libp2p'
import { durations } from '@hoprnet/hopr-utils'
import fs from 'fs'
import { setTimeout } from 'timers/promises'

import { NOISE } from '@chainsafe/libp2p-noise'

const MPLEX = require('libp2p-mplex')

import { default as HoprConnect, type HoprConnectConfig } from '@hoprnet/hopr-connect'
import { Multiaddr } from 'multiaddr'
import pipe from 'it-pipe'
import yargs from 'yargs/yargs'
import { peerIdForIdentity, identityFromPeerId } from './identities'
import type PeerId from 'peer-id'
import type { WriteStream } from 'fs'
import type { PeerStoreType, Stream } from '../src/types'

const TEST_PROTOCOL = '/hopr-connect/test/0.1.0'

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

async function startNode(
  {
    peerId,
    port,
    pipeFileStream
  }: {
    peerId: PeerId
    port: number
    pipeFileStream?: WriteStream
  },
  options: HoprConnectConfig
) {
  console.log(
    `starting node, bootstrap address ${
      options.config?.initialNodes != undefined && options.config?.initialNodes.length > 0
        ? options.config.initialNodes[0].id.toB58String()
        : 'undefined'
    }`
  )

  const node = await libp2p.create({
    peerId,
    addresses: {
      listen: [`/ip4/0.0.0.0/tcp/${port}/p2p/${peerId.toB58String()}`]
    },
    modules: {
      transport: [HoprConnect as any],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE as any]
    },
    config: {
      transport: {
        HoprConnect: options
      },
      peerDiscovery: {
        autoDial: false
      },
      relay: {
        // Conflicts with HoprConnect's own mechanism
        enabled: false
      },
      nat: {
        // Conflicts with HoprConnect's own mechanism
        enabled: false
      }
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
        await setTimeout(durations.seconds(cmdDef.waitForSecs))
        console.log(`finished waiting`)
        break
      }
      case 'dial': {
        const targetPeerId = peerIdForIdentity(cmdDef.targetIdentityName)
        const targetAddress = new Multiaddr(`/ip4/127.0.0.1/tcp/${cmdDef.targetPort}/p2p/${targetPeerId.toB58String()}`)
        console.log(`dialing ${cmdDef.targetIdentityName}`)
        await node.dial(targetAddress)

        console.log(`dialed`)
        break
      }
      case 'msg': {
        const targetPeerId = peerIdForIdentity(cmdDef.targetIdentityName)
        const relayPeerId = peerIdForIdentity(cmdDef.relayIdentityName)

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
        const targetPeerId = peerIdForIdentity(cmdDef.targetIdentityName)
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

function parseCLIOptions() {
  return yargs(process.argv.slice(2))
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
      describe: '[testing] enforce relayed connection, used to NAT behavior',
      type: 'boolean',
      demandOption: true
    })
    .option('noWebRTCUpgrade', {
      describe: '[testing] stick to relayed connection even if WebRTC is available',
      type: 'boolean',
      demandOption: true
    })
    .option('noUPNP', {
      describe: '[testing] do not check UPNP',
      type: 'boolean',
      default: true
    })
    .option('preferLocalAddresses', {
      describe: '[testing] treat local address as public IP addresses',
      type: 'boolean',
      demandOption: true
    })
    .option('runningLocally', {
      describe: '[testing] consider localhost as exposed host',
      type: 'boolean',
      default: false
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
}

async function main() {
  const parsedOpts = parseCLIOptions()

  let bootstrapAddress: PeerStoreType | undefined

  if (parsedOpts.bootstrapPort != null && parsedOpts.bootstrapIdentityName != null) {
    const bootstrapPeerId = peerIdForIdentity(parsedOpts.bootstrapIdentityName)
    bootstrapAddress = {
      id: bootstrapPeerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${parsedOpts.bootstrapPort}/p2p/${bootstrapPeerId.toB58String()}`)]
    }
  }
  const peerId = peerIdForIdentity(parsedOpts.identityName)

  let pipeFileStream: WriteStream | undefined
  if (parsedOpts.pipeFile) {
    pipeFileStream = fs.createWriteStream(parsedOpts.pipeFile)
  }

  console.log(`running node ${parsedOpts.identityName} on port ${parsedOpts.port}`)
  const node = await startNode(
    {
      peerId,
      port: parsedOpts.port,
      pipeFileStream
    },
    {
      config: {
        initialNodes: bootstrapAddress ? [bootstrapAddress] : undefined,
        maxRelayedConnections: parsedOpts.maxRelayedConnections,
        relayFreeTimeout: parsedOpts.relayFreeTimeout
      },
      testing: {
        __noDirectConnections: parsedOpts.noDirectConnections,
        __noWebRTCUpgrade: parsedOpts.noWebRTCUpgrade,
        __noUPNP: parsedOpts.noUPNP,
        __preferLocalAddresses: parsedOpts.preferLocalAddresses,
        __runningLocally: false
      }
    }
  )

  await executeCommands({ node, cmds: parsedOpts.script, pipeFileStream })

  console.log(`all tasks executed`)
}

process.on('unhandledRejection', (error) => {
  console.log('unhandledRejection', error)
  process.exit(1)
})

main()
