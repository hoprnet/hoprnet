import React from 'react'

export default function Connector({
  httpEndpoint,
  setHTTPEndpoint,
  securityToken,
  setSecurityToken,
  wsEndpoint,
  setWsEndpoint
}) {
  return (
    <>
      <div>
        <label>WS Endpoint</label>{' '}
        <input
          name="wsEndpoint"
          placeholder={wsEndpoint}
          value={wsEndpoint}
          onChange={(e) => setWsEndpoint(e.target.value)}
        />
      </div>
      <div>
        <label>HTTP Endpoint</label>{' '}
        <input
          name="httpEndpoint"
          placeholder={httpEndpoint}
          value={httpEndpoint}
          onChange={(e) => setHTTPEndpoint(e.target.value)}
        />
      </div>
      <div>
        <label>Security Token</label>{' '}
        <input
          name="securityToken"
          placeholder={securityToken}
          value={securityToken}
          onChange={(e) => setSecurityToken(e.target.value)}
        />
      </div>
    </>
  )
}
