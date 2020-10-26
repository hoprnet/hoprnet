import React from 'react'
import { sections, elements } from '../components'

const { HeroFull, News, Products, FeaturesTabs, FeaturesTiles, Blogs, Clients, TeamAndInvestors, Contact } = sections
const { HoprCircle } = elements

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <h1>hello</h1>
        <HoprCircle />
        <HeroFull />
        <News id="news" />
        <Products id="products" hasBgColor invertColor redirect />
        <FeaturesTabs id="built_for" redirect />
        <FeaturesTiles id="all_about" hasBgColor invertColor />
        <Blogs id="blogs" redirect />
        <Clients id="investors" hasBgColor invertColor showQuestion />
        <TeamAndInvestors id="team_and_investors" />
        <Contact id="contact" topDivider />
      </React.Fragment>
    )
  }
}

export default Home
