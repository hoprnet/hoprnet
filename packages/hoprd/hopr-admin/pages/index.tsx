import React, { useEffect, useState } from 'react'
import dynamic from 'next/dynamic'
import Head from 'next/head'
import Logo from '../src/components/logo'
import { Logs } from '../src/components/log'
import styles from '../styles/Home.module.css'
import Commands from '../src/commands'
import useAppState from '../src/state'
import { type Log, type Settings, createLog } from '../src/utils'
import { readStreamEvent } from '../src/utils/stream'

// TODO: fix type in refactor
const Jazzicon = dynamic(() => import('../src/components/jazzicon'), { ssr: false }) as any
const GIT_HASH = process.env.NEXT_PUBLIC_GIT_COMMIT

export default function Home() {
  // initialize app state
  const app = useAppState()
  // initialize commands
  const cmds = new Commands(app.api.apiRef.current, () => app.state.aliases)

  // update aliases every 5 seconds
  useEffect(() => {
    const interval = setInterval(() => {
      const api = app.api.apiRef.current
      if (api && app.status === 'CONNECTED') {
        try {
          api
            .getAliases()
            .then((res) => res.json())
            .then(app.updateAliases)
        } catch (error) {
          console.error(error)
        }
      }
    }, 5e3)

    return () => clearInterval(interval)
  }, [app.api.apiRef.current, app.status])

  // store logs
  const [logs, setLogs] = useState<Log[]>([])
  const addLog = (log: Log) => {
    setLogs((prevLogs) => {
      const newLogs = [...prevLogs]
      newLogs.push(log)
      return newLogs
    })
  }

  // toggles connected panel
  const [showConnectedPanel, setShowConnectedPanel] = useState(false)
  const [peers, setPeers] = useState<string[]>([])
  // fetches connected peers
  useEffect(() => {
    const updatePeers = async () => {
      try {
        const api = app.api.apiRef.current
        const peers: {
          connected: {
            peerId: string
          }[]
        } = await api.getPeers().then((res) => res.json())

        setPeers(peers.connected.map((o) => o.peerId))
      } catch (error) {
        console.error(error)
      }
    }

    if (showConnectedPanel) {
      updatePeers()
    }
  }, [showConnectedPanel])

  // toggles settings panel
  const [showSettingsPanel, setShowSettingsPanel] = useState(false)
  const [draftSettings, setDraftSettings] = useState<Settings>(app.state.settings)
  const HandleSettingUpdate = (k: keyof Settings) => (event: React.ChangeEvent<HTMLInputElement>) => {
    setDraftSettings({
      ...draftSettings,
      [k]: event.target.value
    })
  }
  const handleDraftSettingsSave = () => {
    app.updateSettings(draftSettings)
    setShowSettingsPanel(false)
  }

  // handle user inputs
  const [input, setInput] = useState<string>('')
  const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setInput(event.target.value)
  }
  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      event.stopPropagation()
      cmds.execute((msg: string) => addLog(createLog(msg)), input)
      setInput('')
    }
  }

  // attach event listener for new streams events
  const handleStreamEvent = (event: MessageEvent<any>) => {
    const eventLogs = readStreamEvent(event)
    for (const log of eventLogs) {
      addLog(log)
    }
  }
  useEffect(() => {
    const socket = app.streamWS.socketRef.current
    if (!socket) return

    socket.addEventListener('message', handleStreamEvent)

    return () => {
      const socket = app.streamWS.socketRef.current
      if (!socket) return
      socket.removeEventListener('message', handleStreamEvent)
    }
  }, [app.streamWS.socketRef.current, app.streamWS.state.status])

  // attach event listener for new messages
  const handleMessageEvent = (event: MessageEvent<any>) => {
    console.log(event)
  }
  useEffect(() => {
    const socket = app.messagesWS.socketRef.current
    if (!socket) return

    socket.addEventListener('message', handleMessageEvent)

    return () => {
      const socket = app.messagesWS.socketRef.current
      if (!socket) return
      socket.removeEventListener('message', handleMessageEvent)
    }
  }, [app.messagesWS.socketRef.current, app.messagesWS.state.status])

  return (
    <div className={styles.container}>
      <Head>
        <title>HOPR Admin</title>
      </Head>

      <Logo onClick={() => setShowConnectedPanel(!showConnectedPanel)} />
      <h1>
        HOPR Logs - {GIT_HASH ? GIT_HASH : '*'}{' '}
        <span className={styles.cogwheelIcon} onClick={() => setShowSettingsPanel(!showSettingsPanel)}>
          ⚙️
        </span>
      </h1>

      <Logs messages={logs} status={app.status} />

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

      {/* display connected peers panel */}
      {showConnectedPanel && (
        <div className={styles.popup}>
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

      {/* display settings panel */}
      {showSettingsPanel && (
        <div className={styles.popup}>
          <h2>Settings</h2>
          <div className={styles.settings}>
            <div>
              API endpoint: <input value={draftSettings.apiEndpoint} onChange={HandleSettingUpdate('apiEndpoint')} />
            </div>
            <div>
              API token: <input value={draftSettings.apiToken || ''} onChange={HandleSettingUpdate('apiToken')} />
            </div>
            <div>
              <button onClick={handleDraftSettingsSave}>Save</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
