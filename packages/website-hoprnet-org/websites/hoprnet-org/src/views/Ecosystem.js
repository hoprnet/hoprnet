import React from 'react'
import { sections } from '@hoprnet/hopr-website.components'

const { EcosystemHero, Jobs } = sections

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <EcosystemHero />
        <Jobs hasBgColor invertColor forceIsCompany />
      </React.Fragment>
    )
  }
}

export default Home
