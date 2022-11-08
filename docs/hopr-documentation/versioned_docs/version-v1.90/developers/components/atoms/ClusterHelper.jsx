import React from 'react'

export default function ClusterHelper({
  setHTTPEndpoint,
  setWsEndpoint,
  setSecurityToken,
  selectedNode,
  setSelectedNode
}) {
  const CLUSTER_NODES = 5
  const setEndpointsValueUsingIndex = (index) => {
    const BASE_HTTP = 'http://localhost:1330'
    const BASE_WS = 'ws://localhost:1950'
    const DEFAULT_SECURITY_TOKEN = '^^LOCAL-testing-123^^'
    setHTTPEndpoint(BASE_HTTP + index)
    setWsEndpoint(BASE_WS + index)
    setSecurityToken(DEFAULT_SECURITY_TOKEN)
  }

  return (
    <div style={{ display: 'inline-block ' }}>
      Preload Cluster Node -
      {Array.from({ length: 5 }, (_, index) => (
        <button
          style={{
            background: selectedNode == index + 1 && 'blue',
            color: selectedNode == index + 1 && 'white'
          }}
          onClick={() => {
            setSelectedNode(index + 1)
            setEndpointsValueUsingIndex(index + 1)
          }}
        >
          {index + 1}
        </button>
      ))}
    </div>
  )
}
