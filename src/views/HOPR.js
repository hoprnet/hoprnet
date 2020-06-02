import React from 'react'
import Team from '../components/sections/Team'
import Clients from '../components/sections/Clients'
import Investors from '../components/sections/Investors'

class HOPR extends React.Component {
  render() {
    return (
      <React.Fragment>
        <Clients id="clients" />
        <Investors id="investors" topDivider />
        <Team id="team" topDivider />
      </React.Fragment>
    )
  }
}

export default HOPR
