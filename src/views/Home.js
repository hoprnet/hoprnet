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
        <FeaturesTabs topDivider />
        <FeaturesTiles topDivider />
        <Clients topDivider />
        <TeamAndInvestors topDivider />
        <Contact topDivider />
      </React.Fragment>
    )
  }
}

export default Home
