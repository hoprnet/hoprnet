import React, { useState } from 'react';

export default function IframeLoader({ url, loadingMessage = 'Loading...', callback = () => { } }) {
  const [isLoading, setLoading] = useState(true);
  const hasLoaded = () => {
    setLoading(false);
    callback();
  }
  return (<div style={{ marginTop: "20px" }}>
    {isLoading ? (
      <h1>{loadingMessage}</h1>
    ) : null}
    <iframe
      src={url}
      width="100%"
      height="1000"
      onLoad={() => hasLoaded()}
      frameBorder="0"
      marginHeight="0"
      marginWidth="0"
    />
  </div>)
}