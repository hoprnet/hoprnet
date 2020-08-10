import React from 'react'
import { sections } from '@hoprnet/hopr-website.components'

const { NodeHero } = sections

class Node extends React.Component {
  render() {
    return (
      <React.Fragment>
        <NodeHero />
      </React.Fragment>
    )
  }
}

export default Node
