import { Interface, performSTUNRequests } from './stun.js'
import { defer } from '@hoprnet/hopr-utils'
import { Socket as UDPSocket } from 'dgram'
import { Socket as TCPSocket } from 'net'
import { Multiaddr } from '@multiformats/multiaddr'

async function isExposedHost(
  externalInterface: Interface,
  tcpSocket,
  udpSocket: UDPSocket
): Promise<{
  udpMapped: boolean
  tcpMapped: boolean
}> {
  const UDP_TEST = new TextEncoder().encode('TEST_UDP')
  const TCP_TEST = new TextEncoder().encode('TEST_TCP')

  const waitForIncomingUdpPacket = defer<void>()
  const waitForIncomingTcpPacket = defer<void>()

  const TIMEOUT = 500

  const abort = new AbortController()
  const tcpTimeout = setTimeout(() => {
    abort.abort()
    waitForIncomingTcpPacket.reject()
  }, TIMEOUT).unref()
  const udpTimeout = setTimeout(waitForIncomingUdpPacket.reject.bind(waitForIncomingUdpPacket), TIMEOUT).unref()

  const checkTcpMessage = (socket: TCPSocket) => {
    socket.on('data', (data: Buffer) => {
      if (u8aEquals(data, TCP_TEST)) {
        clearTimeout(tcpTimeout)
        waitForIncomingTcpPacket.resolve()
      }
    })
  }
  this.tcpSocket.on('connection', checkTcpMessage)

  const checkUdpMessage = (msg: Buffer) => {
    if (u8aEquals(msg, UDP_TEST)) {
      clearTimeout(udpTimeout)
      waitForIncomingUdpPacket.resolve()
    }
  }
  this.udpSocket.on('message', checkUdpMessage)

  const secondUdpSocket = createSocket('udp4')
  secondUdpSocket.send(UDP_TEST, port, externalIp)

  let done = false
  const cleanUp = (): void => {
    if (done) {
      return
    }
    done = true
    clearTimeout(tcpTimeout)
    clearTimeout(udpTimeout)
    this.udpSocket.removeListener('message', checkUdpMessage)
    this.tcpSocket.removeListener('connection', checkTcpMessage)
    tcpSocket.destroy()
    secondUdpSocket.close()
  }

  const tcpSocket = createConnection({
    port,
    host: externalIp,
    signal: abort.signal
  })
    .on('connect', () => {
      tcpSocket.write(TCP_TEST, (err: any) => {
        if (err) {
          log(`Failed to send TCP packet`, err)
        }
      })
    })
    .on('error', (err: any) => {
      if (err && (err.code == undefined || err.code !== 'ABORT_ERR')) {
        error(`Error while checking NAT situation`, err.message)
      }
    })

  if (!done) {
    const results = await Promise.allSettled([waitForIncomingUdpPacket.promise, waitForIncomingTcpPacket.promise])

    cleanUp()

    return {
      udpMapped: results[0].status === 'fulfilled',
      tcpMapped: results[1].status === 'fulfilled'
    }
  }

  return {
    udpMapped: false,
    tcpMapped: false
  }
}

function checkTCPHairpin() {}

async function checkUDPHairpin(externalInterface: Interface, socket: UDPSocket): Promise<boolean> {
  return (
    (await performSTUNRequests(
      [new Multiaddr(`/ip4/${externalInterface.address}/udp/${externalInterface.port}`)],
      socket,
      undefined
    )) != undefined
  )
}
