import React from 'react'
import { sections } from '@hoprnet/hopr-website.components'

const { Products2, FeaturesTiles, TeamAndInvestors } = sections

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <Products2 id="products" redirect />
        <FeaturesTiles id="all_about" hasBgColor invertColor />
        <TeamAndInvestors id="team_and_investors" />
      </React.Fragment>
    )
  }
}

export default Home
