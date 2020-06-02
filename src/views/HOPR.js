import React from 'react'
import AboutUs from '../components/sections/AboutUs'
import Governance from '../components/sections/Governance'
import Clients from '../components/sections/Clients'
import Investors from '../components/sections/Investors'
import Team from '../components/sections/Team'

class HOPR extends React.Component {
  render() {
    return (
      <React.Fragment>
        <AboutUs id="about" />
        <Governance id="governance" hasBgColor invertColor />
        <Clients id="clients" />
        <Investors id="investors" topDivider />
        <Team id="team" topDivider />
      </React.Fragment>
    )
  }
}

export default HOPR
