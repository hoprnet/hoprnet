import { isExposedHost, handleTcpStunRequest, handleUdpStunRequest, getExternalIp } from './index.js'
import { ip6Lookup } from '../../utils/index.js'

import { Multiaddr } from '@multiformats/multiaddr'

import { type AddressInfo, createServer, type Socket } from 'net'
import { createSocket } from 'dgram'
import assert from 'assert'

async function startServer() {
  const tcpServer = createServer()
  const udpSocket = createSocket({
    type: 'udp6',
    reuseAddr: true,
    lookup: ip6Lookup
  })

  await new Promise<void>((resolve) => tcpServer.listen(resolve))
  const tcpPort = (tcpServer.address() as AddressInfo).port

  await new Promise<void>((resolve) => udpSocket.bind(tcpPort, resolve))

  tcpServer.on('connection', (socket) => handleTcpStunRequest(socket, socket))
  udpSocket.on('message', (msg, rinfo) => handleUdpStunRequest(udpSocket, msg, rinfo))

  return {
    tcpPort,
    close: async () => {
      await new Promise<any>((resolve) => tcpServer.close(resolve))
      await new Promise<void>((resolve) => udpSocket.close(resolve))
    }
  }
}

async function startUdpStunServer() {
  const udpSocket = createSocket({
    type: 'udp6',
    reuseAddr: true,
    lookup: ip6Lookup
  })

  await new Promise<void>((resolve) => udpSocket.bind(undefined, resolve))
  udpSocket.on('message', (msg, rinfo) => handleUdpStunRequest(udpSocket, msg, rinfo))

  return {
    port: udpSocket.address().port,
    close: async () => {
      await new Promise<void>((resolve) => udpSocket.close(resolve))
    }
  }
}

async function startClient(serverPort: number) {
  const tcpServer = createServer()
  const udpSocket = createSocket({
    type: 'udp6',
    reuseAddr: true,
    lookup: ip6Lookup
  })

  await new Promise<void>((resolve) => tcpServer.listen(resolve))
  const tcpPort = (tcpServer.address() as AddressInfo).port

  await new Promise<void>((resolve) => udpSocket.bind(tcpPort, resolve))

  const server = new Multiaddr(`/ip4/127.0.0.1/tcp/${serverPort}`)
  return {
    isExposed: await isExposedHost(
      [server],
      (listener) => {
        const onConnection = (socket: Socket) => {
          listener(socket, socket)
        }
        tcpServer.on('connection', onConnection)

        return () => tcpServer.removeListener('connection', onConnection)
      },
      udpSocket,
      tcpPort
    ),
    close: async () => {
      await new Promise<any>((resolve) => tcpServer.close(resolve))
      await new Promise<void>((resolve) => udpSocket.close(resolve))
    }
  }
}

describe('STUN exposed host check', function () {
  it('check if host is exposed', async function () {
    const { tcpPort, close: closeServer } = await startServer()

    const { isExposed, close: closeClient } = await startClient(tcpPort)

    assert(isExposed == true)

    await closeClient()
    await closeServer()
  })
})

describe('STUN external IP check', function () {
  it('determine external IP address', async function () {
    const { port: portFirst, close: closeFirst } = await startUdpStunServer()
    const { port: portSecond, close: closeSecond } = await startUdpStunServer()

    const servers = [
      new Multiaddr(`/ip4/127.0.0.1/tcp/${portFirst}`),
      new Multiaddr(`/ip4/127.0.0.1/tcp/${portSecond}`)
    ]

    const udpSocket = createSocket({
      type: 'udp6',
      reuseAddr: true,
      lookup: ip6Lookup
    })

    await new Promise<void>((resolve) => udpSocket.bind(undefined, resolve))

    await getExternalIp(servers, udpSocket, true)

    await closeFirst()
    await closeSecond()
    await new Promise<void>((resolve) => udpSocket.close(resolve))
  })
})
