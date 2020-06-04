import React from 'react'
import Enabling from '../components/sections/Enabling'
import OpenSource from '../components/sections/OpenSource'

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
