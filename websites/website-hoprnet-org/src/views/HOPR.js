import React from 'react'
import { sections } from '../components'

const { AboutUs, Governance, Clients, Investors, Token, Values, Team, Jobs, ContactTabs } = sections

class HOPR extends React.Component {
  render() {
    return (
      <React.Fragment>
        <AboutUs id="about" />
        <Governance id="governance" hasBgColor invertColor />
        <Clients id="clients" showQuestion />
        {/* <Investors id="investors" hasBgColor invertColor showQuestion /> */}
        <Token id="token" hasBgColor invertColor />
        <Values id="values" />
        <Team id="team" hasBgColor invertColor />
        <Jobs id="jobs" />
        <ContactTabs id="contact" hasBgColor invertColor redirect />
      </React.Fragment>
    )
  }
}

export default HOPR
