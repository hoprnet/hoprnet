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
        <div className="center-content">
          <div className="container-sm">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Enabling Data Privacy
            </h2>
            <div className="mb-32 hero-figure reveal-from-top" data-reveal-delay="200">
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
            </div>
            <div className="pt-32 reveal-from-top" data-reveal-delay="300">
              We're past the tipping point for privacy. Lack of privacy isn't just a problem for individuals suffering
              at the hands of data harvesting social media giants. Data privacy has become a major societal and economic
              issue requiring government intervention and the creation of entire compliance departments.
              <br />
              <br />
              GDPR, HIPAA, CCPA and other regulation is like a band-aid on a broken leg. At HOPR, we're building the
              foundations for a whole new approach: privacy infrastructure that can be used by corporations and
              individuals to build privacy-first applications and services on the web or blockchain.
              <br />
              <br />
              The HOPR protocol provides network-level and metadata privacy for every kind of data exchange. A mixnet
              protects the identity of both sender and recipient by routing data via relayers who mix traffic and earn
              HOPR tokens for their efforts.
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...props} id="how" hasBgColor invertColor>
        <div className="center-content">
          <div className="container-sm">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              How? The HOPR protocol
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
                HOPR thus provides economic incentives to run a global privacy network sustainably and at scale without
                compromising privacy.
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
