import React from 'react'
import HeroFull from '../components/sections/HeroFull'
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
        <FeaturesTabs id="built_for" hasBgColor invertColor redirect />
        <FeaturesTiles id="all_about" />
        <Blogs id="blogs" hasBgColor invertColor redirect />
        <Clients id="investors" />
        <TeamAndInvestors id="team_and_investors" topDivider />
        <Contact id="contact" />
      </React.Fragment>
    )
  }
}

export default Home
