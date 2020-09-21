import React from 'react'
import { sections } from '../components'

const { Enabling, OpenSource } = sections

class WeAre extends React.Component {
  render() {
    return (
      <React.Fragment>
        <Enabling id="enabling_data_privacy" />
        <OpenSource id="open_source_support" />
      </React.Fragment>
    )
  }
}

export default WeAre
