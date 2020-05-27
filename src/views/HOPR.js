import React from 'react'
import Team from '../components/sections/Team'
import Investors from '../components/sections/Investors'

class HOPR extends React.Component {
  render() {
    return (
      <React.Fragment>
        <Team id="team" />
        <Investors id="investors" topDivider />
      </React.Fragment>
    )
  }
}

export default HOPR
