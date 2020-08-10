import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Image from '../elements/Image'
import { SectionProps } from '../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const NodeHero = props => {
  const oddSections = {
    hasBgColor: props.hasBgColor,
    invertColor: props.invertColor,
  }

  const evenSections = {
    hasBgColor: !oddSections.hasBgColor,
    invertColor: !oddSections.invertColor,
  }

  return (
    <>
      <GenericSection {...oddSections}>
        <div className="center-content">
          <div className="container">
            <p className="m-0 mb-32 reveal-from-top big-title pb-32" data-reveal-delay="300">
              Guardians Of Privacy
            </p>
            <p className="reveal-from-top" data-reveal-delay="350">
              To the HOPR Node Runners, we dedicate this page to you. Online privacy, personal data control, and
              Ethereum blockchain integrity all owe you a debt of gratitude. Although, you’ll get more than just
              gratitude, as you’ll earn upcoming HOPR token incentives (coming Q4!) from our plug-and-earn mixnet node
              PC.
            </p>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...evenSections}>
        <div className="center-content">
          <div className="container" style={{ display: 'flex' }}>
            <div className="mb-32 reveal-from-left" data-reveal-delay="200" style={{ flex: 1 }}>
              <div className="tiles-item-inner">
                <div className="features-tiles-item-header">
                  <div className="features-tiles-item-image mb-16">
                    <Image
                      src={require('../assets/images/icons/github.svg')}
                      alt="Github Logo"
                      width={56}
                      height={56}
                    />
                  </div>
                </div>
                <div className="features-tiles-item-content">
                  <h4 className="mt-0 mb-8">Github</h4>
                  <p className="m-0 text-sm">
                    If you want to run a node without the HOPR Node PC, that’s perfectly acceptable and possible as
                    well! Check out{' '}
                    <a
                      href="https://docs.hoprnet.org/home/getting-started/hopr-chat"
                      className="text-high-color underline"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      docs.hoprnet.org
                    </a>{' '}
                    for details on how to install.
                  </p>
                </div>
              </div>
            </div>
            <div className="mb-32 reveal-from-left" data-reveal-delay="250" style={{ flex: 1 }}>
              <div className="tiles-item-inner">
                <div className="features-tiles-item-header">
                  <div className="features-tiles-item-image mb-16">
                    <Image
                      src={require('../assets/images/icons/with-blue-stroke/server-settings.png')}
                      alt="HOPR Node"
                      width={56}
                      height={56}
                    />
                  </div>
                </div>
                <div className="features-tiles-item-content">
                  <h4 className="mt-0 mb-8">HOPR Hardware Node</h4>
                  <p className="m-0 text-sm">
                    Order our limited supply of initial HOPR Node PCs now! Following our coverage in Coindesk and
                    Cointelegraph, our wait list reached almost 400 people, which surprised even us! However, we are
                    only making an initial 100 node PCs available, so we highly recommend ordering now.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection id="node" {...oddSections}>
        <div className="center-content">
          <div className="container">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="300">
              Specifications
            </h2>
            <p className="reveal-from-top" data-reveal-delay="350">
              The HOPR Node is a pre-configured version of the{' '}
              <a
                href="https://ava.do/avado-i2"
                className="text-high-color underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                AVADO i2
              </a>
              , a plug-and-play blockchain-ready node. It has high specifications (8GB RAM, Intel Pentium 4415) with
              upgraded SSD storage (1TB), making it more than powerful enough to install and run an Ethereum full node.
              The HOPR Node contains the first release of HOPR Alpha, a layer-0 privacy software able to connect to the
              HOPR Network. The HOPR Node is ready to use: just connect it to your router! You can also use it to run
              other kinds of nodes, making it a viable IPFS, Filecoin, or ETH2 node.
            </p>
          </div>
        </div>
      </GenericSection>
      <GenericSection id="checkout" {...evenSections}>
        <div className="center-content">
          <div className="container">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="300">
              Order Checkout Form
            </h2>
            <div className="reveal-from-top" data-reveal-delay="350">
              <iframe
                title="HOPR Defenders of Privacy"
                src="https://docs.google.com/forms/d/e/1FAIpQLSeEM5Mmx-R0JAx164gP9X64QMCFUD-azmfZgyOR0wb1bP8PfA/viewform?embedded=true&hl=en"
                width="700"
                height="650"
                frameBorder="0"
                marginHeight="0"
                marginWidth="0"
              />
            </div>
            <p className="mt-32">
              The HOPR Node PC will ship beginning in September and will take 3 - 4 weeks to arrive.
            </p>
          </div>
        </div>
      </GenericSection>
    </>
  )
}

NodeHero.propTypes = propTypes
NodeHero.defaultProps = defaultProps

export default NodeHero
