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

async function startClient(serverPorts: number[]) {
  const tcpServer = createServer()
  const udpSocket = createSocket({
    type: 'udp6',
    reuseAddr: true,
    lookup: ip6Lookup
  })

  await new Promise<void>((resolve) => tcpServer.listen(resolve))
  const tcpPort = (tcpServer.address() as AddressInfo).port

  await new Promise<void>((resolve) => udpSocket.bind(tcpPort, resolve))

  const servers = serverPorts.map((serverPort: number) => new Multiaddr(`/ip4/127.0.0.1/tcp/${serverPort}`))
  return {
    isExposed: await isExposedHost(
      servers,
      (listener) => {
        const onConnection = (socket: Socket) => {
          listener(socket, socket)
        }
        tcpServer.on('connection', onConnection)

        return () => tcpServer.removeListener('connection', onConnection)
      },
      udpSocket,
      tcpPort,
      false,
      true
    ),
    close: async () => {
      await new Promise<any>((resolve) => tcpServer.close(resolve))
      await new Promise<void>((resolve) => udpSocket.close(resolve))
    }
  }
}

describe('STUN exposed host check', function () {
  it('check if host is exposed', async function () {
    const { tcpPort: firstPort, close: closeFirstServer } = await startServer()
    const { tcpPort: secondPort, close: closeSecondServer } = await startServer()

    const { isExposed, close: closeClient } = await startClient([firstPort, secondPort])

    assert(isExposed == true)

    await closeClient()
    await closeFirstServer()
    await closeSecondServer()
  })
})

describe('STUN external IP check', function () {
  it('determine external IP address', async function () {
    const { port: firstPort, close: closeFirst } = await startUdpStunServer()
    const { port: secondPort, close: closeSecond } = await startUdpStunServer()

    const servers = [
      new Multiaddr(`/ip4/127.0.0.1/tcp/${firstPort}`),
      new Multiaddr(`/ip4/127.0.0.1/tcp/${secondPort}`)
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
