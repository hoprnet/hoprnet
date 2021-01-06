import React, { useEffect, useState, useRef } from 'react'
import styles from '../styles/log.module.css'
import dynamic from 'next/dynamic'

const Jazzicon = dynamic(() => import('../components/jazzicon'), { ssr: false })

export function AbbreviatedId({ id }) {
  let [expanded, setExpanded] = useState(false)
  if (expanded) {
    return <span className={styles.logId}>{id}</span>
  }
  return (
    <abbr className={styles.logId} title={id} onClick={(e) => setExpanded(true)}>
      {id.slice(48)}
    </abbr>
  )
}

export function LogLine(props) {
  console.log(props)
  let raw = props.value.msg
  let msg = []
  let ids = []
  let match

  let lastIndex = 0
  const idRegex = /(\b\w{53})\b/g // NB: Cannot be global variable, has state!

  while ((match = idRegex.exec(raw)) !== null) {
    console.log('>', match, msg)
    ids.push(match[0])
    msg.push(match.input.slice(lastIndex, match.index))
    msg.push(<AbbreviatedId id={match[0]} />)
    lastIndex = match.index + match[0].length
  }
  if (lastIndex != raw.length) {
    msg.push(raw.slice(lastIndex))
  }
  if (msg.length == 0) {
    msg = raw // No matches
  }

  return (
    <div key={props.value.ts} className={styles.logline}>
      <time>{props.value.ts.slice(11)}</time>
      <pre>{msg}</pre>
      <div className={styles.loglineicons}>
        {ids.slice(0, 1).map((x) => (
          <Jazzicon key={x} diameter={15} address={x} />
        ))}
        &nbsp;
      </div>
    </div>
  )
}

export function Logs(props) {
  let container = useRef(null)

  useEffect(() => {
    container.current.scrollIntoView({ block: 'end', behaviour: 'smooth' })
  })

  let cls = styles.logs + ' ' + (props.connecting ? styles.connecting : '')
  return (
    <div className={cls}>
      <div ref={container}>
        {props.connecting && <span>Attempting to connect</span>}
        {props.messages.map((x) => (
          <LogLine value={x} />
        ))}
      </div>
    </div>
  )
}
