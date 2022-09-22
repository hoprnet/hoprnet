import { type Log, createLog } from '.'

/**
 * Legacy event reader from stream socket.
 * Read incoming stream event and convert it to a client log.
 * @param event Event coming from '/api/v2/node/stream/websocket'
 * @returns log readable by hopr-admin
 */
export const readStreamEvent = (event: MessageEvent<any>): Log | undefined => {
  if (event.data === undefined) {
    return undefined
  }

  try {
    const data: {
      type: string
      msg: string
      ts: string
    } = JSON.parse(event.data)

    // we ignore plain messages, instead print HOPRd logs
    if (data.type === 'message') return undefined
    // we ignore connected type, which is simply a line of all connected peers
    if (data.type === 'connected') return undefined
    // we are only interested in messages which have a message
    if (!data.msg) return undefined

    return createLog(data.msg, +new Date(data.ts))
  } catch (error) {
    console.log('error reading stream log', error)
  }
}
