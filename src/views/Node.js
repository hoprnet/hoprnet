import React from 'react'
import NodeHero from '../components/sections/NodeHero'
import RequestTestnet from '../components/sections/RequestTestnet'

class Node extends React.Component {
  render() {
    return (
      <React.Fragment>
        <NodeHero />
        <RequestTestnet hasBgColor invertColor />
      </React.Fragment>
    )
  }
}

export default Node
