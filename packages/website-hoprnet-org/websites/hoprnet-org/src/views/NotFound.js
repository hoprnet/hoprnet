import React from 'react'
import { sections } from '@hoprnet/hopr-website.components'

const { NotFound } = sections

class WeAre extends React.Component {
  render() {
    return (
      <React.Fragment>
        <NotFound hasBgColor invertColor />
      </React.Fragment>
    )
  }
}

export default WeAre
