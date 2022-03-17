// import Cookies from 'js-cookie'
import React, { useEffect, useState } from 'react'
import dynamic from 'next/dynamic'
import Head from 'next/head'
import Logo from '../src/components/logo'
import { Logs } from '../src/components/log'
// import TokenInput from '../src/components/tokenInput'
import styles from '../styles/Home.module.css'
// import { Connection } from '../src/connection'
import Commands from '../src/commands'
import useAppState from '../src/state'
import type { Log } from '../src/utils'

const Jazzicon = dynamic(() => import('../src/components/jazzicon'), { ssr: false })
const GIT_HASH = process.env.NEXT_PUBLIC_GIT_COMMIT

export default function Home() {
  // initialize app state
  const app = useAppState()
  // initialize commands
  const cmds = new Commands(app.api.apiRef.current)

  // store logs
  const [logs, setLogs] = useState<Log[]>([])
  const addLog = (log: Log) => {
    logs.push(log)
    setLogs(logs)
  }

  // toggles connected panel
  const [showConnectedPanel, setShowConnectedPanel] = useState(false)
  const [peers, setPeers] = useState<string[]>([])
  // fetches connected peers
  useEffect(() => {
    const updatePeers = async () => {
      const api = app.api.apiRef.current
      const peers: {
        connected: {
          peerId: string
        }[]
      } = await api.getPeers()

      setPeers(peers.connected.map((o) => o.peerId))
    }

    if (showConnectedPanel) {
      updatePeers()
    }
  }, [showConnectedPanel])

  // handle user inputs
  const [input, setInput] = useState<string>('')
  const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setInput(event.target.value)
  }
  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      event.stopPropagation()
      cmds.execute((msg) => {
        addLog({
          msg,
          id: String(Math.random()),
          ts: +new Date()
        })
      }, input)
      setInput('')
    }
  }

  return (
    <div className={styles.container}>
      <Head>
        <title>HOPR Admin</title>
      </Head>

      <Logo onClick={() => setShowConnectedPanel(!showConnectedPanel)} />
      <h1>HOPR Logs - {GIT_HASH ? GIT_HASH : '*'}</h1>

      <Logs messages={logs} connecting={app.status !== 'CONNECTED'} authRequired={false} />

      <div className="send">
        <input
          id="command"
          type="text"
          autoFocus
          placeholder="type 'help' for full list of commands"
          value={input}
          onChange={handleInputChange}
          onKeyDown={handleKeyDown}
        />
      </div>

      {/* {(authFailed || cookie === null) && <TokenInput handleTokenSet={handleTokenSet} />} */}

      {/* display connected peers panel */}
      {showConnectedPanel && (
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
