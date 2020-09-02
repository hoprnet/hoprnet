import React, { useEffect, useState, useRef } from "react";
import styles from '../styles/log.module.css'

export function LogLine(props){
  return (
    <div className={styles.logline}>
      <time>{ props.value.ts.slice(11) }</time>
      <pre><code>{ props.value.msg }</code></pre>
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
