import React, { useEffect, useState, KeyboardEvent } from 'react'
import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Logo from '../components/logo'
import { Logs } from '../components/log'
import { Connection } from '../connection'
import dynamic from 'next/dynamic'
import Cookies from 'js-cookie'
import { render } from 'react-dom'

const Jazzicon = dynamic(() => import('../components/jazzicon'), { ssr: false })

class TokenInput extends React.Component {
  constructor(props) {
    super(props)
    this.handleKeyPress = this.handleKeyPress.bind(this)
  }

  handleKeyPress(e) {
    if (e.key == 'Enter') {
      var text = e.target.value
      Cookies.set('X-Auth-Token', text)
      this.props.handleTokenSet()
    }
  }

  render() {
    const tokenCookie = Cookies.get('X-Auth-Token')
    return tokenCookie === undefined ? (
      <div className="send">
        <input
          className="token"
          onKeyPress={this.handleKeyPress}
          id="token"
          type="password"
          placeholder="security token"
        />
      </div>
    ) : null
  }
}

export default function Home() {
  let connection

  const [showConnected, setShowConnected] = useState(false)
  const [connecting, setConnecting] = useState(true)
  const [ready, setReady] = useState(false)
  const [messages, setMessages] = useState([]) // The fetish for immutability in react means this will be slower than a mutable array..
  const [peers, setConnectedPeers] = useState([])
  const [, updateState] = React.useState()
  const handleAuthFailed = React.useCallback(() => {
    Cookies.remove('X-Auth-Token')
    setAuthFailed(true)
  }, [])
  const [authFailed, setAuthFailed] = useState(false)
  const handleTokenSet = React.useCallback(() => {
    connection.connect()
    setAuthFailed(false)
    updateState({})
  }, [])

  useEffect(() => {
    if (typeof window !== 'undefined') {
      connection = new Connection(setConnecting, setReady, setMessages, setConnectedPeers, handleAuthFailed)
      return Connection.disconnect
    }
  }, [])

  const cookie = Cookies.get('X-Auth-Token')

  return (
    <div className={styles.container}>
      <Head>
        <title>HOPR Admin</title>
      </Head>

      <Logo onClick={() => setShowConnected(!showConnected)} />
      <h1>HOPR Logs</h1>

      <Logs messages={messages} connecting={connecting} authRequired={authFailed} />

      <div className="send">
        <input id="command" type="text" autoFocus placeholder="type 'help' for full list of commands" />
      </div>

      {(authFailed || cookie === null) && <TokenInput handleTokenSet={handleTokenSet} />}

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
