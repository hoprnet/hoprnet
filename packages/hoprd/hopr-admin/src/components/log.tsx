import type { Log } from '../utils'
import React, { useEffect, useState, useRef } from 'react'
import styles from '../../styles/log.module.css'
import dynamic from 'next/dynamic'

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

const getAbbreviatedIds = (text: string, regex: RegExp): [string, any][] => {
  const textArray: string[] = text.split(regex)
  return textArray.map<[string, any | undefined]>((str) => {
    if (regex.test(str)) {
      return [str, <AbbreviatedId id={str} key={str} />]
    } else {
      return [str, undefined]
    }
  })
}

export function LogLine({ log }: { log: Log }) {
  const lines = getAbbreviatedIds(log.msg, PEERID_REGEXP)
  const ids = lines.filter(([, elem]) => !!elem).map(([str]) => str)
  const output = lines.map(([str, elem]) => elem || str)

  return (
    <div key={log.ts} className={styles.logline}>
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

export function Logs(props: { connecting: boolean; messages: Log[] }) {
  let container = useRef(null)

  useEffect(() => {
    container.current.scrollIntoView({ block: 'end', behaviour: 'smooth' })
  })

  let cls = styles.logs + ' ' + (props.connecting ? styles.connecting : '')

  return (
    <div className={cls}>
      <div ref={container}>
        {props.connecting && <span>Attempting to connect, please ensure correct settings are set!</span>}
        {props.messages.map((log) => (
          <LogLine log={log} key={log.id} />
        ))}
      </div>
    </div>
  )
}
