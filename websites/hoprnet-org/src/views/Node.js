import React from 'react'
import { sections } from '@hoprnet/hopr-website.components'

const { NodeHero, RequestTestnet } = sections

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
