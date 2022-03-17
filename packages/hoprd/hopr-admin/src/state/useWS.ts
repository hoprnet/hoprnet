/*
  A react hook.
  Keeps websocket connection alive, reconnects on disconnections or endpoint change.
*/
import { useEffect, useRef, useState } from 'react'
import { debounce } from 'lodash'
import { type Settings, type ApiPath, isSSR } from '../utils'

export type ConnectionStatus = 'CONNECTED' | 'DISCONNECTED'

const useWebsocket = (settings: Settings, path: ApiPath) => {
  // update timestamp when you want to reconnect to the websocket
  const [reconnectTmsp, setReconnectTmsp] = useState<number>()
  const [state, setState] = useState<{
    status: ConnectionStatus
    error?: string
  }>({ status: 'DISCONNECTED' })

  const socketRef = useRef<WebSocket>()

  const setReconnectTmspDebounced = debounce((timestamp: number) => {
    setReconnectTmsp(timestamp)
  }, 5e3)

  const handleOpenEvent = () => {
    console.info('WS CONNECTED')
    setState((draft) => {
      draft.status = 'CONNECTED'
      return draft
    })
  }

  const handleCloseEvent = () => {
    console.info('WS DISCONNECTED')
    setState((draft) => {
      draft.status = 'DISCONNECTED'
      return draft
    })
    setReconnectTmspDebounced(+new Date())
  }

  const handleErrorEvent = (e: Event) => {
    console.error('WS ERROR', e)
    setState((draft) => {
      draft.status = 'DISCONNECTED'
      draft.error = String(e)
      return draft
    })
    setReconnectTmspDebounced(+new Date())
  }

  // runs everytime "endpoint" or "reconnectTmsp" changes
  useEffect(() => {
    if (isSSR) return

    // disconnect from previous connection
    if (socketRef.current) {
      console.info('WS Disconnecting..')
      socketRef.current.close(1000, 'Shutting down')
    }

    // need to set the token in the query parameters, to enable websocket authentication
    const wsUrl = new URL(settings.apiEndpoint + path)
    wsUrl.protocol = 'ws'
    if (settings.apiToken) {
      wsUrl.search = `?apiToken=${settings.apiToken}`
    }
    console.info('WS Connecting..')
    socketRef.current = new WebSocket(wsUrl)

    // handle connection opening
    socketRef.current.addEventListener('open', handleOpenEvent)
    // handle connection closing
    socketRef.current.addEventListener('close', handleCloseEvent)
    // handle errors
    socketRef.current.addEventListener('error', handleErrorEvent)

    // cleanup when unmounting
    return () => {
      if (!socketRef.current) return

      socketRef.current.removeEventListener('open', handleOpenEvent)
      socketRef.current.removeEventListener('close', handleCloseEvent)
      socketRef.current.removeEventListener('error', handleErrorEvent)
    }
  }, [settings.apiEndpoint, settings.apiToken, reconnectTmsp])

  return {
    state,
    socketRef
  }
}

export default useWebsocket
