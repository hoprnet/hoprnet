import React, { useEffect, useState } from 'react'
import WebSocketHandler from './WebSocketHandler'
import Connector from './atoms/Connector'
import ClusterHelper from './atoms/ClusterHelper'
import { getHeaders } from './utils'

export default function RPSGame() {
  const [securityToken, setSecurityToken] = useState('')
  const [selectedNode, setSelectedNode] = useState();
  const [wsEndpoint, setWsEndpoint] = useState('ws://localhost:3000')
  const [httpEndpoint, setHTTPEndpoint] = useState('http://localhost:3001')
  const [address, setAddress] = useState('')
  const [isReferee, setIsReferee] = useState()
  const [referee, setReferee] = useState('')
  const [notification, setNotification] = useState('')

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

  const sendMove = async (move) => {
    if (!address) return
    await fetch(`${httpEndpoint}/api/v2/messages`, {
      method: 'POST',
      headers: getHeaders(securityToken, true),
      body: JSON.stringify({
        recipient: referee,
        body: `${address}-${move}`
      })
    }).catch((err) => console.error(err))
    setNotification(`You have sent the move ${move} to referee ${referee}`)
  }

  return (
    <div>
      <ClusterHelper
        selectedNode={selectedNode}
        setSelectedNode={setSelectedNode}
        setHTTPEndpoint={setHTTPEndpoint}
        setWsEndpoint={setWsEndpoint}
        setSecurityToken={setSecurityToken}
      />
      <br />
      <span>PeerId: {address}</span>
      <Connector
        httpEndpoint={httpEndpoint}
        setHTTPEndpoint={setHTTPEndpoint}
        wsEndpoint={wsEndpoint}
        setWsEndpoint={setWsEndpoint}
        securityToken={securityToken}
        setSecurityToken={setSecurityToken}
      />
      <div>
        <div style={{ display: 'inline-block', marginRight: '10px' }}>
          <label htmlFor="isReferee">Is Referee</label>
          <input
            onChange={(e) => setIsReferee(e.target.checked)}
            id="isReferee"
            type="checkbox"
          />
        </div>{''}
        <label>Referee</label>{' '}
        <input
          name="referee"
          disabled={isReferee}
          placeholder={referee}
          value={referee}
          onChange={(e) => setReferee(e.target.value)}
        />
      </div>
      {address && !isReferee &&
        <>
          <button disabled={!referee} onClick={() => sendMove('paper')}>Send "paper" move</button>
          <button disabled={!referee} onClick={() => sendMove('scissors')}>Send "scissors" move</button>
          <button disabled={!referee} onClick={() => sendMove('rock')}>Send "rock" move</button>
        </>
      }
      {notification && <><br />{notification}</>}
      <><br /><WebSocketHandler wsEndpoint={wsEndpoint} securityToken={securityToken} multipleMessages={isReferee} /></>
    </div >
  )
}
