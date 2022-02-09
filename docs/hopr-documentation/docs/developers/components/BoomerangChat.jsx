import React, { useEffect, useState } from 'react'
import WebSocketHandler from './WebSocketHandler'

export default function BoomerangChat() {
  const [message, setMessage] = useState('Hello world')
  const [securityToken, setSecurityToken] = useState('')
  const [wsEndpoint, setWsEndpoint] = useState('ws://localhost:3000')
  const [httpEndpoint, setHTTPEndpoint] = useState('http://localhost:3001')
  const [address, setAddress] = useState('')

  const getHeaders = (isPost = false) => {
    const headers = new Headers()
    if (isPost) {
      headers.set('Content-Type', 'application/json')
      headers.set('Accept-Content', 'application/json')
    }
    headers.set('Authorization', 'Basic ' + btoa(securityToken))
    return headers
  }

  useEffect(() => {
    const loadAddress = async () => {
      const headers = getHeaders()
      const account = await fetch(`${httpEndpoint}/api/v2/account/addresses`, {
        headers
      })
        .then((res) => res.json())
        .catch((err) => console.error(err))
      setAddress(account?.hopr)
    }
    loadAddress()
  }, [securityToken, httpEndpoint])

  const sendMessage = async () => {
    if (!address) return
    await fetch(`${httpEndpoint}/api/v2/messages`, {
      method: 'POST',
      headers: getHeaders(true),
      body: JSON.stringify({
        recipient: address,
        body: message
      })
    }).catch((err) => console.error(err))
  }

  return (
    <div>
      <div>
        <label>WS Endpoint</label>{' '}
        <input
          name="wsEndpoint"
          placeholder={wsEndpoint}
          value={wsEndpoint}
          onChange={(e) => setWsEndpoint(e.target.value)}
        />
      </div>
      <div>
        <label>HTTP Endpoint</label>{' '}
        <input
          name="httpEndpoint"
          placeholder={httpEndpoint}
          value={httpEndpoint}
          onChange={(e) => setHTTPEndpoint(e.target.value)}
        />
      </div>
      <div>
        <label>Security Token</label>{' '}
        <input
          name="securityToken"
          placeholder={securityToken}
          value={securityToken}
          onChange={(e) => setSecurityToken(e.target.value)}
        />
      </div>
      <div>
        <label>Send a message</label>{' '}
        <input name="httpEndpoint" value={message} placeholder={message} onChange={(e) => setMessage(e.target.value)} />
      </div>
      <button onClick={() => sendMessage()}>Send message to node</button>
      <br />
      <br />
      <WebSocketHandler wsEndpoint={wsEndpoint} securityToken={securityToken} />
    </div>
  )
}
