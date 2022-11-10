import React, { useEffect, useState } from 'react'
import WebSocketHandler from './WebSocketHandler'
import Connector from './atoms/Connector'
import { getHeaders } from './utils'

export default function BoomerangChat() {
  const [message, setMessage] = useState('Hello world')
  const [securityToken, setSecurityToken] = useState('')
  const [wsEndpoint, setWsEndpoint] = useState('ws://localhost:3000')
  const [httpEndpoint, setHTTPEndpoint] = useState('http://localhost:3001')
  const [address, setAddress] = useState('')

  useEffect(() => {
    const loadAddress = async () => {
      const headers = getHeaders(securityToken)
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
      headers: getHeaders(securityToken, true),
      body: JSON.stringify({
        recipient: address,
        body: message
      })
    }).catch((err) => console.error(err))
  }

  return (
    <div>
      <Connector
        httpEndpoint={httpEndpoint}
        setHTTPEndpoint={setHTTPEndpoint}
        wsEndpoint={wsEndpoint}
        setWsEndpoint={setWsEndpoint}
        securityToken={securityToken}
        setSecurityToken={setSecurityToken}
      />
      <div>
        <label>Send a message</label>{' '}
        <input name="message" value={message} placeholder={message} onChange={(e) => setMessage(e.target.value)} />
      </div>
      <button onClick={() => sendMessage()}>Send message to node</button>
      <br />
      <br />
      <WebSocketHandler wsEndpoint={wsEndpoint} securityToken={securityToken} />
    </div>
  )
}
