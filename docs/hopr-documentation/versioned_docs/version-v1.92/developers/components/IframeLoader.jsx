import React, { useState, useRef } from 'react'

export default function IframeLoader({ url, loadingMessage = 'Loading...', callback = () => {}, ...props }) {
  const [isLoading, setLoading] = useState(true)
  const iframe = useRef(null)

  const hasLoaded = () => {
    setLoading(false)
    console.log('IFRAME (from loader)', iframe)
    callback(iframe.current)
  }
  return (
    <div style={{ marginTop: '20px' }}>
      {isLoading ? <h1>{loadingMessage}</h1> : null}
      <iframe ref={iframe} {...props} src={url} onLoad={() => hasLoaded()} />
    </div>
  )
}
