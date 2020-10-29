import React, {useEffect, useState} from 'react'
import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Logo from '../components/logo'
import {Logs} from '../components/log'
import {Connection} from '../connection'
import dynamic from 'next/dynamic'

const Jazzicon = dynamic(() => import('../components/jazzicon'), {ssr: false})

export default function Home() {
  let connection

  const [showConnected, setShowConnected] = useState(false)
  const [connecting, setConnecting] = useState(true)
  const [messages, setMessages] = useState([]) // The fetish for immutability in react means this will be slower than a mutable array..
  const [peers, setConnectedPeers] = useState([])

  useEffect(() => {
    if (typeof window !== 'undefined') {
      connection = new Connection(setConnecting, setMessages, setConnectedPeers)
      return Connection.disconnect
    }
  }, [])

  return (
    <div className={styles.container}>
      <Head>
        <title>HOPR Admin</title>
      </Head>

      <Logo onClick={() => setShowConnected(!showConnected)} />
      <h1>HOPR Logs [TESTNET NODE]</h1>

      <Logs messages={messages} connecting={connecting} />

      <div className="send">
        <input
          id="command"
          type="text"
          disabled={connecting}
          autoFocus
          placeholder="type 'help' for full list of commands"
        />
      </div>

      {showConnected && (
        <div className={styles.connectedPeers}>
          <h2>Connected Peers ({peers.length})</h2>
          <div className={styles.connectedPeersList}>
            {peers.map((x) => (
              <div className={styles.peer} key={x}>
                <Jazzicon diameter={40} address={x} className={styles.peerIcon} />
                <div>{x}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
