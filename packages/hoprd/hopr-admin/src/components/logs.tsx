import React, { useEffect, useState, useRef } from 'react'
import dynamic from 'next/dynamic'

import styles from '../../styles/log.module.css'
import type { Log } from '../utils/index'

const DISCLAIMER = `ATTENTION: This software is still under development. For testing, this node requires both native and HOPR funds. Use the command "balance" to check the node's funds.`

// TODO: fix type in refactor
const Jazzicon = dynamic(() => import('./jazzicon'), { ssr: false }) as any

export function AbbreviatedId({ id }: { id: string }) {
  let [expanded, setExpanded] = useState(false)
  if (expanded) {
    return <span className={styles.logId}>{id}</span>
  }
  return (
    <abbr className={styles.logId} title={id} onClick={() => setExpanded(true)}>
      {id.slice(48)}
    </abbr>
  )
}

const PEERID_REGEXP = /(\b\w{53})\b/g // NB: Cannot be global variable, has state!

const getAbbreviatedIds = (log: Log, regex: RegExp): [string, any][] => {
  const textArray: string[] = log.msg.split(regex)
  let peerIdsFound = 0
  return textArray.map<[string, any | undefined]>((str) => {
    if (regex.test(str)) {
      return [str, <AbbreviatedId id={str} key={`${log.id}-${++peerIdsFound}`} />]
    } else {
      return [str, undefined]
    }
  })
}

export function LogLine({ log }: { log: Log }) {
  const lines = getAbbreviatedIds(log, PEERID_REGEXP)
  const ids = lines.filter(([, elem]) => !!elem).map(([str]) => str)
  const output = lines.map(([str, elem]) => elem || str)

  return (
    <div key={log.id} className={styles.logline}>
      <time>{new Date(log.ts).toISOString().slice(11)}</time>
      <pre>{output}</pre>
      <div className={styles.loglineicons}>
        {ids.slice(0, 1).map((x) => (
          <Jazzicon key={x} diameter={15} address={x} />
        ))}
        &nbsp;
      </div>
    </div>
  )
}

export default function Logs(props: { isConnected: boolean; messages: Log[] }) {
  let container = useRef(null)

  useEffect(() => {
    container.current.scrollIntoView({ block: 'end', behaviour: 'smooth' })
  })

  let disclaimer

  if (process.env.NODE_ENV === 'production') {
    disclaimer = <div className={styles.disclaimer}>{DISCLAIMER}</div>
  }

  return (
    <div className={`${styles.logs} ${!props.isConnected ? styles.connecting : ''}`}>
      {disclaimer}
      <div ref={container}>
        {!props.isConnected && <span>Attempting to connect, please ensure correct settings are set!</span>}
        {props.messages.map((log) => (
          <LogLine log={log} key={log.id} />
        ))}
      </div>
    </div>
  )
}
