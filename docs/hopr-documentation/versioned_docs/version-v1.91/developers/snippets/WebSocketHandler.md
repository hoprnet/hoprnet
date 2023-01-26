---
id: WebSocketHandler.jsx
hide_title: true
---

```jsx title="WebSocketHandler.jsx"
import React, { useEffect, useState } from 'react'
import useWebsocket from '../hooks/useWebSockets '

export const WebSocketHandler = ({ wsEndpoint, securityToken }) => {
  const [message, setMessage] = useState('')
  const websocket = useWebsocket({ wsEndpoint, securityToken })
  const { socketRef } = websocket
  const handleReceivedMessage = async (ev) => {
    try {
      // we are only interested in messages, not all the other events coming in on the socket
      const data = JSON.parse(ev.data)
      console.log('WebSocket Data', data)
      if (data.type === 'message') {
        setMessage(data.msg)
      }
    } catch (err) {
      console.error(err)
    }
  }
  useEffect(() => {
    if (!socketRef.current) return
    socketRef.current.addEventListener('message', handleReceivedMessage)

    return () => {
      if (!socketRef.current) return
      socketRef.current.removeEventListener('message', handleReceivedMessage)
    }
  }, [socketRef.current])

  return <span>{message ? message : 'You have no messages.'}</span>
}

export default WebSocketHandler
```
