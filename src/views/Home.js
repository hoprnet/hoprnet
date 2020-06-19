import React from 'react'
import HeroFull from '../components/sections/HeroFull'
import Products from '../components/sections/Products'
import FeaturesTabs from '../components/sections/FeaturesTabs'
import FeaturesTiles from '../components/sections/FeaturesTiles'
import Blogs from '../components/sections/Blogs'
import Clients from '../components/sections/Clients'
import TeamAndInvestors from '../components/sections/TeamAndInvestors'
import Contact from '../components/sections/Contact'

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <HeroFull />
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
