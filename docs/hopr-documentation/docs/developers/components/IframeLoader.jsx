import React, { useState } from 'react'

export default function IframeLoader({ url, loadingMessage = 'Loading...', callback = () => { }, ...props }) {
  const [isLoading, setLoading] = useState(true)
  const hasLoaded = () => {
    setLoading(false)
    callback()
  }
  return (
    <div style={{ marginTop: '20px' }}>
      {isLoading ? <h1>{loadingMessage}</h1> : null}
      <iframe
        {...props}
        src={url}
        onLoad={() => hasLoaded()}
      />
    </div>
  )
}
