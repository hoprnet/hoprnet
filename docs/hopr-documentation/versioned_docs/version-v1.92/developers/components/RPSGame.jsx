import React, { useEffect, useState } from 'react'
import WebSocketHandler from './WebSocketHandler'
import Connector from './atoms/Connector'
import ClusterHelper from './atoms/ClusterHelper'
import { getHeaders } from './utils'

export default function RPSGame() {
  const [securityToken, setSecurityToken] = useState('')
  const [selectedNode, setSelectedNode] = useState()
  const [wsEndpoint, setWsEndpoint] = useState('ws://localhost:3000')
  const [httpEndpoint, setHTTPEndpoint] = useState('http://localhost:3001')
  const [messages, setMessages] = useState([])
  const [address, setAddress] = useState('')
  const [isReferee, setIsReferee] = useState()
  const [referee, setReferee] = useState('')
  const [notification, setNotification] = useState('')

  const SCISSORS_MOVE = 'scissors'
  const ROCK_MOVE = 'rock'
  const PAPER_MOVE = 'paper'

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

  const sendMessage = async (recipient, body) => {
    if (!address) return
    await fetch(`${httpEndpoint}/api/v2/messages`, {
      method: 'POST',
      headers: getHeaders(securityToken, true),
      body: JSON.stringify({
        recipient,
        body
      })
    }).catch((err) => console.error(err))
  }

  const sendMove = async (move) => {
    await sendMessage(referee, `${address}-${move}`)
    setNotification(`You have sent the move ${move} to referee ${referee}`)
  }

  useEffect(() => {
    // Game logic goes here, when messages are received.
    const gameLogic = async () => {
      const [player1, player2] = messages
        .slice(messages.length - 2)
        .map((move) => ({ address: move.split('-')[0], move: move.split('-')[1] }))

      // We ignore all other messages.
      if (!player1 || !player2) return
      if (!player1.move || !player2.move) return

      if (player1.address != player2.address) {
        if (
          (player1.move == ROCK_MOVE && player2.move == ROCK_MOVE) ||
          (player1.move == ROCK_MOVE && player2.move == ROCK_MOVE) ||
          (player1.move == ROCK_MOVE && player2.move == ROCK_MOVE)
        ) {
          await sendMessage(
            player1.address,
            `You tied with ${player2.address}: [1] ${player1.move}, [2] ${player2.move}`
          )
          await sendMessage(
            player2.address,
            `You tied with ${player1.address}: [1] ${player1.move}, [2] ${player2.move}`
          )
        } else if (
          (player1.move == ROCK_MOVE && player2.move == SCISSORS_MOVE) ||
          (player1.move == SCISSORS_MOVE && player2.move == PAPER_MOVE) ||
          (player1.move == PAPER_MOVE && player2.move == ROCK_MOVE)
        ) {
          await sendMessage(
            player1.address,
            `You won! ${player2.address} lost: [1] ${player1.move}, [2] ${player2.move}`
          )
          await sendMessage(
            player2.address,
            `You lost... ${player1.address} won: [1] ${player1.move}, [2] ${player2.move}`
          )
        } else {
          await sendMessage(
            player2.address,
            `You won! ${player1.address} lost: [1] ${player1.move}, [2] ${player2.move}`
          )
          await sendMessage(
            player1.address,
            `You lost... ${player2.address} won: [1] ${player1.move}, [2] ${player2.move}`
          )
        }
      }
    }
    isReferee && gameLogic()
  }, [messages])

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
          <input onChange={(e) => setIsReferee(e.target.checked)} id="isReferee" type="checkbox" />
        </div>
        {''}
        <label>Referee</label>{' '}
        <input
          name="referee"
          disabled={isReferee}
          placeholder={referee}
          value={referee}
          onChange={(e) => setReferee(e.target.value)}
        />
      </div>
      {address && !isReferee && (
        <>
          <button disabled={!referee} onClick={() => sendMove(PAPER_MOVE)}>
            Send "paper" move
          </button>
          <button disabled={!referee} onClick={() => sendMove(SCISSORS_MOVE)}>
            Send "scissors" move
          </button>
          <button disabled={!referee} onClick={() => sendMove(ROCK_MOVE)}>
            Send "rock" move
          </button>
        </>
      )}
      {notification && (
        <>
          <br />
          {notification}
        </>
      )}
      <>
        <br />
        <WebSocketHandler
          wsEndpoint={wsEndpoint}
          securityToken={securityToken}
          multipleMessages={isReferee}
          messages={messages}
          setMessages={setMessages}
        />
      </>
    </div>
  )
}
