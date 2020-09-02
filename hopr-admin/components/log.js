import React, { useEffect, useState, useRef } from "react";

export function LogLine(props){
  return props.value
}


export function Logs(props){
  let container = useRef(null)

  useEffect(() => {
    container.current.scrollIntoView({block: 'end', behaviour: 'smooth'});
  })

  let cls = props.connecting ? 'logs connecting' : 'logs'
  return (
      <div className={cls}>
        <pre>
          <code id='log' ref={container}>
          { props.messages.map(x => <LogLine value={x} />) }
          </code>
        </pre>
      </div>
  )
}
