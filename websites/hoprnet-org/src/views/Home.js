import React from 'react'
import { sections } from '@hoprnet/hopr-website.components'

const { HeroFull, News, Products, FeaturesTabs, FeaturesTiles, Blogs, Clients, TeamAndInvestors, Contact } = sections

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <HeroFull />
        <News id="news" />
        <Products id="products" hasBgColor invertColor redirect />
        <FeaturesTabs id="built_for" redirect />
        <FeaturesTiles id="all_about" hasBgColor invertColor />
        <Blogs id="blogs" redirect />
        <Clients id="investors" hasBgColor invertColor />
        <TeamAndInvestors id="team_and_investors" />
        <Contact id="contact" />
      </React.Fragment>
    )
  }
}

export default Home
