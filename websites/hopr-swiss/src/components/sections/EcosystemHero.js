import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Button from '../elements/Button'
import Image from '../elements/Image'
import Tabs, { TabList, Tab } from '../elements/Tabs'
import { SectionProps } from '../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const Ecosystem = props => {
  const oddSections = {
    hasBgColor: props.hasBgColor,
    invertColor: props.invertColor,
  }

  const evenSections = {
    hasBgColor: !oddSections.hasBgColor,
    invertColor: !oddSections.invertColor,
  }

  return (
    <div className="ecosystem">
      <GenericSection {...oddSections} className="cards">
        <div className="center-content">
          <div className="container">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              We want YOU to grow!
            </h2>

            <div className="cards-container">
              <div className="mb-32 reveal-from-left" data-reveal-delay="200" style={{ backgroundColor: '#003A4B' }}>
                <a href="#personal">
                  <div className="tiles-item-inner">
                    <div className="features-tiles-item-header">
                      <p className="mt-0 mb-24 text-sm">Personal</p>
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/love-heart-keyhole@140x140.png')}
                          alt="Heart Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </div>
                    <div className="features-tiles-item-content">
                      <h4 className="mt-0 mb-8">Hackathons + Community</h4>
                      <p className="m-0 text-sm">
                        We're organizing events to grow our network and give back to the Community
                        <br />
                        (coming mid-July 2020).
                      </p>
                    </div>
                  </div>
                </a>
              </div>

              <div className="mb-32 reveal-from-left" data-reveal-delay="200" style={{ backgroundColor: '#005A73' }}>
                <a href="#professional">
                  <div className="tiles-item-inner">
                    <div className="features-tiles-item-header">
                      <p className="mt-0 mb-24 text-sm">Professional</p>
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/building-modern@140x140.png')}
                          alt="Modern Building Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </div>
                    <div className="features-tiles-item-content">
                      <h4 className="mt-0 mb-8">Your company</h4>
                      <p className="m-0 text-sm">
                        HOPR is looking for partners who want to benefit from our network. If data privacy is important
                        to you, get in touch.
                      </p>
                    </div>
                  </div>
                </a>
              </div>
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...evenSections} id="personal" className="personal">
        <div className="center-content">
          <div className="container">
            <div className="header section-header reveal-from-top" data-reveal-delay="150">
              <div className="features-tiles-item-image">
                <Image
                  src={require('../assets/images/icons/love-heart-keyhole@140x140.png')}
                  alt="Heart Icon"
                  width={56}
                  height={56}
                />
              </div>
              <p className="mt-0 mb-0 h4 ml-24">Personal</p>
            </div>

            <div className="pb-32">
              The HOPR network is built by the community, for the community - in a maximally open and participatory
              fashion. Therefore, we are looking for the brightest minds to build the vision of HOPR together with us.
              Our community involvement program will continue to grow and prosper but this is how you get started now:
            </div>

            <div className="cards-container mt-32">
              <div style={{ backgroundColor: '#53A3B9' }}>
                <h4>Beach Level:</h4>
                Tackle our technical bounties on{' '}
                <a href="/layer0-data-privacy#bounties" className="underline">
                  Gitcoin
                </a>{' '}
                and receive $ + HOPR Token
              </div>
              <div style={{ backgroundColor: '#164856' }}>
                <h4>Alpine Level:</h4>
                Hack your project and get rewarded during an upcoming HOPR Hackathon{' '}
                <a href="/layer0-data-privacy#bounties" className="underline">
                  Gitcoin
                </a>
              </div>
              <div style={{ backgroundColor: 'white', color: 'rgb(22, 72, 86)' }}>
                <h4 style={{ color: 'rgb(22, 72, 86)' }}>Moon Level:</h4>
                More exciting opportunities: Coming up September 2020
              </div>
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...oddSections} id="professional" className="professional">
        <div className="center-content">
          <div className="container">
            <div className="header section-header reveal-from-top" data-reveal-delay="150">
              <div className="features-tiles-item-image">
                <Image
                  src={require('../assets/images/icons/building-modern@140x140.png')}
                  alt="Modern Building Icon"
                  width={56}
                  height={56}
                />
              </div>
              <p className="mt-0 mb-0 h4 ml-24">Professional</p>
              (existing business)
            </div>
            <div className="pb-32">
              If your company cares about data privacy, get in touch with us. We're looking to partner with a limited
              number of companies, particularly in medtech fields. We guarantee a personal bespoke service at the level
              that best suits your business. Here's what you can expect from us:
            </div>
            What you can expect from us:
            <Tabs className="mt-24 blogs-tabs blogs-tabs-inner">
              <TabList>
                <Tab tabId="tab-a" className="is-active">
                  <div className="blogs-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="50">
                    <Image src={require('../assets/images/icons/ambulance-call@140x140.png')} alt="Call Icon" />
                  </div>
                  <p>(Level 3)</p>
                  <div className="fw-700">Personal Contact</div>
                  <p>Connect with our development team and receive a personal manager for all your questions</p>
                </Tab>
                <Tab tabId="tab-b" className="is-active">
                  <div className="blogs-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="100">
                    <Image src={require('../assets/images/icons/server-settings@140x140.png')} alt="Lock Icon" />
                  </div>
                  <p>(Level 2)</p>
                  <div className="fw-700">HOPR Node</div>
                  <p>The association will provide you with a free HOPR node</p>
                </Tab>
                <Tab tabId="tab-c" className="is-active">
                  <div className="blogs-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="150">
                    <Image
                      src={require('../assets/images/icons/monetization-sponsor@140x140.png')}
                      alt="Programming Team Icon"
                    />
                  </div>
                  <p>(Level 1)</p>
                  <div className="fw-700">HOPR Token Grant</div>
                  <p>The association will issue you a grant paid in HOPR tokens</p>
                </Tab>
              </TabList>
            </Tabs>
            <Button
              color={oddSections.invertColor ? 'secondary' : 'primary'}
              tag="a"
              href="https://docs.google.com/forms/d/e/1FAIpQLSeh9nZ20aKVUpGg8_hEY0DbAJeQDog9c0aMehfwaGXT91ezXA/viewform?usp=sf_link&hl=en"
              target="_blank"
              rel="noopener noreferrer"
            >
              Apply now for company program
            </Button>
          </div>
        </div>
      </GenericSection>
    </div>
  )
}

Ecosystem.propTypes = propTypes
Ecosystem.defaultProps = defaultProps

export default Ecosystem
