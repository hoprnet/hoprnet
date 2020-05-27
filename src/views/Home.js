import React from 'react'
import HeroFull from '../components/sections/HeroFull'
import Clients from '../components/sections/Clients'
import FeaturesTabs from '../components/sections/FeaturesTabs'
import FeaturesTiles from '../components/sections/FeaturesTiles'
import Contact from '../components/sections/Contact'

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <HeroFull />
        <FeaturesTabs topDivider />
        <FeaturesTiles topDivider />
        <Clients topDivider />
        <Contact />
      </React.Fragment>
    )
  }
}

export default Home
