import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Image from '../elements/Image'
import Button from '../elements/Button'
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
            <p className="reveal-from-top" data-reveal-delay="400">
              HOPR makes no profit from the first-of-its-kind custom node PC. Instead, we thank you for helping ensure a
              decentralized Web3 future, where everyone has control of their own data.
            </p>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...evenSections}>
        <div className="center-content">
          <div className="container node-hero-features-tiles-container">
            <div className="mb-32 reveal-from-left" data-reveal-delay="200">
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
                  <h4 className="mt-0 mb-8">Run Your Own Metal</h4>
                  <p className="m-0 text-sm">
                    If you want to run a node on your own hardware without the HOPR Node PC, that’s perfectly acceptable
                    and possible as well! Check out our{' '}
                    <a
                      href="https://docs.hoprnet.org/home/getting-started/hopr-chat"
                      className="text-color-high underline"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      GitBook instructions
                    </a>{' '}
                    for details on how to install.
                  </p>
                </div>
              </div>
            </div>
            <div className="mb-32 reveal-from-left" data-reveal-delay="250">
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
                  <h4 className="mt-0 mb-8">Your HOPR Hardware Node</h4>
                  <p className="m-0 text-sm">
                    Order our limited supply of initial HOPR Node PCs now! Following our coverage in{' '}
                    <a
                      href="https://www.coindesk.com/binance-labs-leads-1m-seed-round-in-crypto-tor-alternative-hopr"
                      className="text-color-high underline"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      Coindesk
                    </a>{' '}
                    and{' '}
                    <a
                      href="https://cointelegraph.com/news/hopr-data-privacy-testnet-to-launch-following-investment-by-binance"
                      className="text-color-high underline"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      Cointelegraph
                    </a>
                    , our wait list reached several hundred, which surprised even us! However, we are only making an
                    initial 100 node PCs available, so we highly recommend ordering now.
                  </p>
                  <Button
                    tag="a"
                    color="primary"
                    className="mt-32"
                    href="https://ava.do/checkout/hopr"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    ORDER NOW
                  </Button>
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
              Specifications of HOPR Node PC
            </h2>
            <p className="reveal-from-top" data-reveal-delay="350">
              The HOPR Node PC is a pre-configured version of the{' '}
              <a
                href="https://ava.do/avado-i2"
                className="text-color-high underline"
                target="_blank"
                rel="noopener noreferrer"
              >
                AVADO i2
              </a>
              , a plug-and-play blockchain-ready node. It has high specifications (8GB RAM, Intel Pentium 4415) with
              upgraded SSD storage (1TB), making it more than powerful enough to install and run an Ethereum full node.
              The HOPR Node PC contains the first release of HOPR Alpha, a layer-0 privacy software able to connect to
              the HOPR Network. The HOPR Node PC is ready to use: just connect it to your router! You can also use it to
              run other kinds of nodes, making it a viable IPFS, Filecoin, or ETH2 node.
            </p>
          </div>
        </div>
      </GenericSection>
      {/* <GenericSection id="checkout" {...evenSections}>
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
      </GenericSection> */}
    </>
  )
}

NodeHero.propTypes = propTypes
NodeHero.defaultProps = defaultProps

export default NodeHero
