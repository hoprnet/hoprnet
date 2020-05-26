import React from 'react'
import HeroFull from '../components/sections/HeroFull'
import Clients from '../components/sections/Clients'
import FeaturesTabs from '../components/sections/FeaturesTabs'
import FeaturesTiles from '../components/sections/FeaturesTiles'
// import Testimonial from '../components/sections/Testimonial'
// import Pricing from '../components/sections/Pricing'
// import Cta from '../components/sections/Cta'
import Team from '../components/sections/Team'
import Investors from '../components/sections/Investors'
import Contact from '../components/sections/Contact'

class Home extends React.Component {
  render() {
    return (
      <React.Fragment>
        <HeroFull /> {/* className="illustration-section-01" */}
        <FeaturesTabs />
        <FeaturesTiles topDivider />
        <Clients topDivider bottomDivider />
        <Team />
        <Investors />
        {/* <Testimonial
          topDivider
          bottomOuterDivider
          className="gradient-section"
        /> */}
        {/* <Pricing topDivider pricingSlider className="has-bg-color-cut" /> */}
        {/* <Cta hasBgColor invertColor split /> */}
        <Contact />
      </React.Fragment>
    )
  }
}

export default Home
