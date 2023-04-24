import type { Connection } from '@libp2p/interface-connection'
import type { Components } from '@libp2p/interfaces/components'
import { CustomEvent } from '@libp2p/interfaces/events'

import { debug } from '../process/index.js'

const log = debug('hopr-core:libp2p:connection')

export async function safeCloseConnection(
  conn: Connection,
  components: Components,
  onErrorFun: (err: Error) => void
): Promise<void> {
  try {
    // await result to prevent race conditions of other logic using this
    // connection which we are trying to close
    await conn.close()
  } catch (err) {
    onErrorFun(err)
  }

  // Verify that the connection has been removed from the connection manager.
  // If not, which could be a bug, trigger the removal manually.
  const existingConnections = components.getConnectionManager().getConnections(conn.remotePeer)
  const storedConn = existingConnections.find((c) => c.id === conn.id)

  // If the connection is still stored, trigger the event.
  if (storedConn) {
    log(`Trigger onDisconnect event for connection ${conn.id} to ${conn.remotePeer.toString()} manually`)
    // @ts-ignore
    components.getConnectionManager().onDisconnect(
      new CustomEvent<Connection>('connectionEnd', {
        detail: conn
      })
    )
  }
}
