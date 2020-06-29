import React from 'react'
import { sections } from '@hoprnet/hopr-website.components'

const { HeroFull, Products, FeaturesTabs, FeaturesTiles, Blogs, TeamAndInvestors } = sections

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <HeroFull />
        <Products id="products" hasBgColor invertColor redirect />
        <FeaturesTabs id="built_for" redirect />
        <FeaturesTiles id="all_about" hasBgColor invertColor />
        <Blogs id="blogs" redirect />
        <TeamAndInvestors id="team_and_investors" hasBgColor invertColor />
      </React.Fragment>
    )
  }
}

export default Home
