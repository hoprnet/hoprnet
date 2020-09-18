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

const Enabling = props => {
  return (
    <>
      <GenericSection {...props}>
        <div className="center-content whole-page">
          <div className="container-sm">
            <p className="section-header mt-0 mb-0 reveal-from-top big-title" data-reveal-delay="150">
              Enabling Data Privacy
            </p>
            {/* <div className="mb-32 hero-figure reveal-from-top" data-reveal-delay="200">
              <Image
                className="has-shadow"
                src={require('../assets/images/Web3-Data-Privacy.png')}
                alt="image of Web3 data privacy and protection"
                width={896}
                height={504}
                style={{
                  borderRadius: '15px',
                }}
              />
            </div> */}
            <div className="pt-32 reveal-from-top" data-reveal-delay="300">
              Everything we do, we get tracked–often without even knowing it. And this data is then used to influence us
              and our decisions. We believe that everyone should have the chance to make their own decisions in their
              lives, as freely and uninfluenced as possible. HOPR has set out to protect your privacy, data, and
              ultimately, your identity.
              <br />
              <br />
              The speed of innovation is increasing on a daily basis and so is the amount of data harvested about us. We
              have the dream that digitalization can improve our lives without costing us all of our privacy. We have
              the dream that everyone can own their personal data again.
              <br />
              <br />
              Your data, your decision.
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...props} id="how" hasBgColor invertColor>
        <div className="center-content">
          <div className="container-sm">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              How? The HOPR Protocol
            </h2>
            <div className="container reveal-from-top" data-reveal-delay="300">
              <div className="column">
                The HOPR protocol provides network-level and metadata privacy for every kind of data exchange. A mixnet
                protects the identity of both sender and recipient by routing data via multiple intermediate relay hops
                that mix traffic.
                <br />
                <br />
                Payments are handled via probabilistic micropayments, our custom layer-2 scaling solution on top of the
                Ethereum blockchain.
              </div>
              <div className="column">
                Relay mix nodes are rewarded for their work in HOPR tokens. Our proof-of-relay mechanism protects
                everyone from dishonest behaviour.
                <br />
                <br />
                HOPR thus provides economic incentives to run a global privacy network sustainably - and at scale -
                without compromising privacy.
              </div>
            </div>
          </div>
        </div>
      </GenericSection>
    </>
  )
}

Enabling.propTypes = propTypes
Enabling.defaultProps = defaultProps

export default Enabling
