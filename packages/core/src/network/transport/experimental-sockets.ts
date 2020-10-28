import net from 'net'

import dgram from 'dgram'

async function main() {
  const tcpServer = net.createServer((socket) => {
    console.log('connection over tcp')

    socket.on('data', (data: Buffer) => {
      console.log(`received over tcp`, new TextDecoder().decode(data))
    })
  })

  const udp4Server = dgram.createSocket({
    type: 'udp4',
    reuseAddr: true
  })

  const udp6Server = dgram.createSocket({
    type: 'udp6',
    reuseAddr: true
  })

  udp4Server.on('message', (msg, _info) => {
    console.log(`received over udp4`, new TextDecoder().decode(msg))
  })

  udp6Server.on('message', (msg, _info) => {
    console.log(`received over udp6`, new TextDecoder().decode(msg))
  })

  const listeningPromise = Promise.all([
    new Promise((resolve) => tcpServer.once('listening', resolve)),
    new Promise((resolve) => udp4Server.once('listening', resolve)),
    new Promise((resolve) => udp6Server.once('listening', resolve))
  ])

  tcpServer.listen(9091)
  udp4Server.bind(9091)
  udp6Server.bind(9091)

  await listeningPromise

  const tcpSocket = net.connect({ port: 9091 }, () => {
    tcpSocket.write(new TextEncoder().encode('sending tcp'))
  })

  const udp4Socket = dgram.createSocket('udp4')

  udp4Socket.connect(9091, '127.0.0.1', () => {
    udp4Socket.send(new TextEncoder().encode('sending udp4'), 9091)
  })

  const udp6Socket = dgram.createSocket('udp6')

  udp6Socket.connect(9091, '127.0.0.1', () => {
    udp6Socket.send(new TextEncoder().encode('sending udp6'), 9091)
  })

  await new Promise((resolve) => setTimeout(resolve, 2000))

  console.log(`after timeout`)
  tcpSocket.destroy()
  udp4Socket.close()
  udp6Socket.close()

  tcpSocket.destroy()
  udp4Server.close()
  udp6Server.close()
}

main()
