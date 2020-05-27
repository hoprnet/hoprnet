import React from 'react'
import HeroFull from '../components/sections/HeroFull'
import FeaturesTabs from '../components/sections/FeaturesTabs'
import FeaturesTiles from '../components/sections/FeaturesTiles'
import Clients from '../components/sections/Clients'
import TeamAndInvestors from '../components/sections/TeamAndInvestors'
import Contact from '../components/sections/Contact'

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <HeroFull />
        <FeaturesTabs id="built_for" topDivider />
        <FeaturesTiles id="all_about" topDivider />
        <Clients id="investors" topDivider />
        <TeamAndInvestors id="team_and_investors" topDivider />
        <Contact id="contact" topDivider />
      </React.Fragment>
    )
  }
}

export default Home
