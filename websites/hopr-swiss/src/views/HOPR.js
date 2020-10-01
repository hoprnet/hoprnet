import React from 'react'
import { sections } from '../components'

const { AboutUs, Values, Team, Jobs, ContactTabs } = sections

class HOPR extends React.Component {
  render() {
    return (
      <React.Fragment>
        <AboutUs id="about" />
        <Values id="values" hasBgColor invertColor />
        <Team id="team" />
        <Jobs id="jobs" hasBgColor invertColor />
        <ContactTabs id="contact" redirect />
      </React.Fragment>
    )
  }
}

export default HOPR
