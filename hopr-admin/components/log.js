import React, { useEffect, useState, useRef } from "react";
import styles from '../styles/log.module.css'

const ID_REGEX = /(\w{53})/g

export function LogLine(props){
  let raw = props.value.msg
  let msg = []
  let match

  let lastIndex = 0
  while ((match = ID_REGEX.exec(raw)) !== null){
    console.log(">> match", match)
    msg.push(match.input.slice(lastIndex, match.index))
    msg.push(<abbr title={match[0]}>{match[0].slice(48)}</abbr>)
    lastIndex = match.index + match[0].length
  }
  if (msg.length == 0) {
    msg = raw // No matches
  }
  console.log("!!", msg)

  return (
    <div className={styles.logline}>
      <time>{ props.value.ts.slice(11) }</time>
      <pre>{ msg }</pre>
    </div>
  )
}


export function Logs(props){
  let container = useRef(null)

  useEffect(() => {
    container.current.scrollIntoView({block: 'end', behaviour: 'smooth'});
  })

  let cls = styles.logs + (props.connecting ? styles.connecting : '')
  return (
      <div className={cls}>
        <div ref={container}>
        { props.messages.map(x => <LogLine value={x} />) }
        </div>
      </div>
  )
}
