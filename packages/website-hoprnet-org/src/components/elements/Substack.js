import React from 'react'

export default () => (
  <div className="iframe-container" style={{ height: '200px' }}>
    <iframe
      title="substack"
      src="https://hopr.substack.com/embed"
      className="mj-w-res-iframe"
      scrolling="no"
      frameBorder="0"
      marginHeight="0"
      marginWidth="0"
      style={{
        border: '1px solid #EEE',
        background: 'white',
        width: '100%',
        height: '100%',
      }}
    />
  </div>
)
