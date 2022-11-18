import { Interface, performSTUNRequests } from './stun.js'
import { type Socket as UDPSocket } from 'dgram'
import { type Server as TCPSocket, createConnection } from 'net'
import { Multiaddr } from '@multiformats/multiaddr'
import debug from 'debug'

import { u8aEquals } from '@hoprnet/hopr-utils'

// @ts-ignore
import { retimer } from 'retimer'

const log = debug('hopr-connect:hairpin')
const error = debug('hopr-connect:hairpin:error')

export async function checkForHairpinning(
  externalInterface: Interface,
  tcpSocket: TCPSocket,
  udpSocket: UDPSocket
): Promise<{ udpMapped: boolean; tcpMapped: boolean }> {
  const [udpMapped, tcpMapped] = await Promise.all([
    checkUDPHairpin(externalInterface, udpSocket),
    checkTCPHairpin(externalInterface, tcpSocket)
  ])

  return {
    udpMapped,
    tcpMapped
  }
}

async function checkTCPHairpin(externalInterface: Interface, socket: TCPSocket): Promise<boolean> {
  return new Promise<boolean>((resolve) => {
    const TCP_TEST = new TextEncoder().encode('TEST_TCP')

    const TIMEOUT = 500

    const abort = new AbortController()

    const timeout = retimer(() => {
      abort.abort()
      done(undefined, false)
    }, TIMEOUT).unref()

    const checkMessage = (socket: TCPSocket) => {
      socket.on('data', (data: Buffer) => {
        if (u8aEquals(data, TCP_TEST)) {
          timeout.clear()
          done(undefined, true)
        }
      })
    }

    const done = (err: any, result?: boolean) => {
      socket.removeListener('connection', checkMessage)

      resolve(err != undefined ? false : result ?? false)
    }

    socket.on('connection', checkMessage)

    const outgoingSocket = createConnection({
      port: externalInterface.port,
      host: externalInterface.address,
      signal: abort.signal
    })
      .on('connect', () => {
        outgoingSocket.write(TCP_TEST, (err: any) => {
          if (err) {
            log(`Failed to send TCP packet`, err)
          }
        })
      })
      .on('error', (err: any) => {
        if (err && (err.code == undefined || err.code !== 'ABORT_ERR')) {
          error(`Error while checking NAT situation`, err.message)
          done(err)
        }
      })
  })
}

async function checkUDPHairpin(externalInterface: Interface, socket: UDPSocket): Promise<boolean> {
  return (
    (await performSTUNRequests(
      [new Multiaddr(`/ip4/${externalInterface.address}/udp/${externalInterface.port}`)],
      socket,
      undefined
    )) != undefined
  )
}
