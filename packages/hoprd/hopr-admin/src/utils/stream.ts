import { type Log, createLog } from '.'

/**
 * Legacy event reader from stream socket.
 * Read incoming stream event and convert it to client logs.
 */
export const readStreamEvent = (event: any): Log[] => {
  if (event.data === undefined) {
    return []
  }

  const logs: Log[] = []

  try {
    const msg: {
      type: string
      content: string
      timestamp: string
    } = JSON.parse(event.data)

    switch (msg.type) {
      case 'log':
        logs.push(createLog(msg.content, +new Date(msg.timestamp)))
        break
      case 'fatal-error':
        logs.push(createLog(msg.content, +new Date(msg.timestamp)))
        break
      case 'auth-failed':
        logs.push(createLog(msg.content, +new Date(msg.timestamp)))
        break
    }

    return logs
  } catch (error) {
    console.log('error reading stream log', error)
  }
}
