import { once, type EventEmitter } from 'events'

interface Listening<ListenOpts> extends EventEmitter {
  listen: (opts: ListenOpts) => void
}

export async function waitUntilListening<ListenOpts>(socket: Listening<ListenOpts>, port: ListenOpts) {
  const promise = once(socket, 'listening')

  socket.listen(port)

  return promise
}

interface Closing extends EventEmitter {
  close: () => void
}

export async function stopNode(socket: Closing) {
  const closePromise = once(socket, 'close')

  socket.close()

  return closePromise
}
