import React, { useEffect, useState } from 'react'
import dynamic from 'next/dynamic'
import Head from 'next/head'
import Logo from '../src/components/logo'
import Logs from '../src/components/logs'
import styles from '../styles/Home.module.css'
import Commands from '../src/commands'
import useAppState from '../src/state'
import { type Log, type Configuration, createLog, HealthStatus } from '../src/utils'

// TODO: fix type in refactor
const Jazzicon = dynamic(() => import('../src/components/jazzicon'), { ssr: false }) as any
const GIT_HASH = process.env.NEXT_PUBLIC_GIT_COMMIT

export default function Home() {
  // store logs
  const [logs, setLogs] = useState<Log[]>([])
  const addLog = (log: Log) => {
    setLogs((prevLogs) => {
      const newLogs = [...prevLogs]
      newLogs.push(log)
      return newLogs
    })
  }

  // initialize app state
  const app = useAppState(addLog)
  // initialize commands
  const cmds = new Commands(app.api.apiRef.current, {
    getCachedAliases() {
      return app.state.aliases
    },
    updateAliasCache(fn) {
      return app.updateAliases(fn)
    },
    getSymbols() {
      // TODO: fetch from API once supported
      const native = 'xDAI'
      const hopr = 'mHOPR'

      return {
        native,
        hopr,
        nativeDisplay: `NATIVE (${native})`,
        hoprDisplay: `HOPR (${hopr})`
      }
    }
  })

  const updateAliases = async () => {
    const api = app.api.apiRef.current
    if (api && app.streamWS.state.status === 'CONNECTED') {
      try {
        const aliasesResp = await api.getAliases()
        if (aliasesResp.ok) {
          const aliases = await aliasesResp.json()
          app.updateAliases(() => aliases)
        } else {
          console.error(`failed to get aliases with HTTP status: ${aliasesResp.status}`)
        }
      } catch (error) {
        console.error(error)
      }
    }
  }

  const updateHealthStatus = async () => {
    const api = app.api.apiRef.current
    if (api && app.streamWS.state.status === 'CONNECTED') {
      try {
        const infoResp = await api.getInfo()
        if (infoResp.ok) {
          const info = await infoResp.json()
          const status = info.connectivityStatus
          setHealthStatus(HealthStatus[status])
        } else {
          console.error(`failed to get info with HTTP status: ${infoResp.status}`)
        }
      } catch (error) {
        console.error(error)
      }
    }
  }
  // health status indicator
  const [healthStatus, setHealthStatus] = useState<HealthStatus>(HealthStatus.Unknown)
  const [showTooltip, setShowTooltip] = useState(false)
  // update aliases once and every 5 seconds
  // update health status once and every 10 seconds
  useEffect(() => {
    updateAliases()
    updateHealthStatus()

    const aliasesInterval = setInterval(updateAliases, 5e3)
    const healthStatusInterval = setInterval(updateHealthStatus, 10e3)
    return () => {
      clearInterval(aliasesInterval)
      clearInterval(healthStatusInterval)
    }
  }, [app.api.apiRef.current, app.streamWS.state.status === 'CONNECTED'])

  // toggles connected panel
  const [showConnectedPanel, setShowConnectedPanel] = useState(false)
  const [peers, setPeers] = useState<string[]>([])
  // fetches connected peers
  useEffect(() => {
    const updatePeers = async () => {
      try {
        const api = app.api.apiRef.current
        const peersResp = await api.getPeers()
        if (peersResp.ok) {
          const fetchedPeers: {
            connected: {
              peerId: string
            }[]
          } = await peersResp.json()
          setPeers(fetchedPeers.connected.map((o) => o.peerId))
        } else {
          console.error(`failed to get peers with HTTP status: ${peersResp.status}`)
        }
      } catch (error) {
        console.error(error)
      }
    }

    if (showConnectedPanel) {
      updatePeers()
    }
  }, [showConnectedPanel])

  // toggles config panel
  const [showConfigPanel, setShowConfigPanel] = useState(false)
  const [draftConfig, setDraftConfig] = useState<Configuration>(app.state.config)
  const HandleConfigUpdate = (k: keyof Configuration) => (event: React.ChangeEvent<HTMLInputElement>) => {
    setDraftConfig({
      ...draftConfig,
      [k]: event.target.value
    })
  }
  const handleDraftConfigSave = () => {
    app.updateConfig((prevConfig) => ({
      ...prevConfig,
      ...draftConfig
    }))
    setShowConfigPanel(false)
  }

  // handle user inputs
  const [input, setInput] = useState<string>('')
  const [history, setHistory] = useState<{
    history: string[]
    index: number
  }>({
    history: [],
    index: 0 // index 0 stands for no selection in history
  })
  const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setInput(event.target.value)
  }
  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') {
      event.stopPropagation()
      cmds.execute((msg: string) => addLog(createLog(msg)), input)
      setHistory((prevHistory) => {
        if (!input) return prevHistory
        const newHistory = prevHistory.history.slice(0)
        newHistory.unshift(input)

        return {
          history: newHistory.slice(0, 50),
          index: 0
        }
      })
      setInput('')
    } else if (event.key === 'ArrowDown') {
      if (history.index > 0) {
        const newIndex = --history.index

        if (newIndex === 0) {
          setInput('')
        } else {
          setInput(history.history[history.index])
        }

        setHistory((prevHistory) => {
          return {
            ...prevHistory,
            index: newIndex
          }
        })
      }
    } else if (event.key === 'ArrowUp') {
      if (history.index < history.history.length) {
        const newIndex = ++history.index
        const newInput = history.history[history.index - 1]

        setInput(newInput)
        setHistory((prevHistory) => {
          return {
            ...prevHistory,
            index: newIndex
          }
        })
      }
    }
  }

  return (
    <div className={styles.container}>
      <Head>
        <title>HOPR Admin</title>
      </Head>

      <Logo onClick={() => setShowConnectedPanel(!showConnectedPanel)} />
      <h1>
        HOPR Logs - {GIT_HASH ? GIT_HASH : '*'}{' '}
        <div className={styles.titleIcons}>
          <span className={styles.cogwheelIcon} onClick={() => setShowConfigPanel(!showConfigPanel)}>
            ⚙️
          </span>
          <span
            className={styles.healthIndicator}
            onMouseEnter={() => setTimeout(() => setShowTooltip(true), 300)}
            onMouseLeave={() => setTimeout(() => setShowTooltip(false), 300)}
          >
            {healthStatus}
          </span>
          {showTooltip && <span className={styles.healthIndicatorTooltip}>network health indicator</span>}
        </div>
      </h1>

      <Logs messages={logs} isConnected={app.streamWS.state.status === 'CONNECTED'} />

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
          <h2>Peers seen in the network ({peers.length})</h2>
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

      {/* display config panel */}
      {showConfigPanel && (
        <div className={styles.popup}>
          <h2>Configuration</h2>
          <div className={styles.configuration}>
            <div>
              API endpoint: <input value={draftConfig.apiEndpoint} onChange={HandleConfigUpdate('apiEndpoint')} />
            </div>
            <div>
              API token: <input value={draftConfig.apiToken || ''} onChange={HandleConfigUpdate('apiToken')} />
            </div>
            <div>
              <button onClick={handleDraftConfigSave}>Save</button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
