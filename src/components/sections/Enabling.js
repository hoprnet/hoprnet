import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Image from '../elements/Image'
import { SectionProps } from '../../utils/SectionProps'

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
          <div className="container-ms">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Enabling Data Privacy:
            </h2>
            <div className="mb-32 hero-figure reveal-from-top" data-reveal-delay="200">
              <Image
                className="has-shadow"
                src={require('../../assets/images/hopr-illustration-small.png')}
                alt="Hero"
                width={896}
                height={504}
                style={{
                  borderRadius: '15px',
                }}
              />
            </div>
            <div className="pt-32 reveal-from-top" data-reveal-delay="300">
              Today we are past the tipping point of privacy. The absence of privacy is not just an issue for private
              individuals who suffer from data harvesting social media giants. Data privacy is major issue that requires
              governmental interventions and compliance departments to reduce the impact of privacy.
              <br />
              <br />
              GDPR, HIIPA, CCPA and other legal ordinances are attempting to retrospectively fix a broken underlying
              privacy landscape. With HOPR we aim at changing this approach: We are building privacy infrastructure that
              can be leveraged by corporations and private individuals alike, to build privacy-first applications and
              services on the web or on-chain.
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...props} hasBgColor invertColor>
        <div className="pt-32 center-content reveal-from-top" data-reveal-delay="300">
          This is how we do it: The HOPR protocol
          <br />
          <br />
          At HOPR we focus on network-level and metadata privacy. To that end we develop the HOPR protocol which
          provides privacy for any sort of data exchange.
          <br />
          <br />
          HOPR is a mixnet that protects sender and recipient of a data packet beyond end-to-end encryption. This
          privacy is established by indirectly routing data via multiple intermediate relay hops that mix traffic.
          <br />
          <br />
          Relay mix nodes get rewarded for their work by getting paid in HOPR tokens. Our proof-of-relay mechanism
          protects everyone from dishonest node operators extracting funds. The payments are handled via probabilistic
          micropayments, our custom layer-2 scaling solution ontop of the Ethereum blockchain.
          <br />
          <br />
          Thus HOPR provides economic incentives to run a global privacy network sustainably and at scale without
          compromising on primacy.
        </div>
      </GenericSection>
    </>
  )
}

Enabling.propTypes = propTypes
Enabling.defaultProps = defaultProps

export default Enabling
