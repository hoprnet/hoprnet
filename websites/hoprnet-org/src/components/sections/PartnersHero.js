import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
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
    <div className="partners">
      <GenericSection {...oddSections}>
        <div className="center-content">
          <div className="container">
            <h2 className="m-0 mb-32 reveal-from-top big-title pb-32" data-reveal-delay="150">
              Next Generation Data Privacy
            </h2>

            <p className="m-0 mb-32 reveal-from-top" data-reveal-delay="200">
              The HOPR protocol is built for everyone. Our mission of introducing network-level data privacy for all
              industries and individuals starts with you.
              <br />
              <br />
              In becoming our valued partner, we provide you with:
              <br />
              <br />
              <ol className="centered-list">
                <li>Your personal project manager</li>
                <li>A dedicated engineer for the integration of the HOPR protocol</li>
                <li>Membership in the HOPR Association</li>
              </ol>
            </p>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...evenSections} id="advantage">
        <div className="center-content">
          <div className="container">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              The HOPR Advantage
            </h2>

            <div className="reveal-from-top" data-reveal-delay="200">
              <iframe
                title="The HOPR Advantage"
                width="800"
                height="500"
                src="https://www.youtube-nocookie.com/embed/vb7mD8dp11Q"
                frameBorder="0"
                allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
                allowFullScreen
              />
            </div>
          </div>
        </div>
      </GenericSection>
      <GenericSection {...oddSections} id="protocol">
        <div className="center-content">
          <div className="container-sm">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              The HOPR Protocol
            </h2>
            <div className="container reveal-from-top" data-reveal-delay="200">
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
      <GenericSection {...evenSections} id="using">
        <div className="center-content">
          <div className="container">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Become a Partner
            </h2>

            <p className="m-0 mb-32 reveal-from-top" data-reveal-delay="200">
              Your customers, your clients, your business, deserves comprehensive, network-level data privacy. Working
              with the HOPR Protocol ensures that.
              <br />
              <br />
              The next generation starts now.
              <br />
              <br />
              Interested?
            </p>

            <Button
              className="reveal-from-top"
              data-reveal-delay="250"
              color="primary"
              tag="a"
              href="https://docs.google.com/forms/d/e/1FAIpQLSfpw9alXYGO4WvWS8HTF-5keGk_OFshrIIcGzifhfuWs7IN7g/viewform"
              target="_blank"
              rel="noopener noreferrer"
              style={{ fontSize: '25px' }}
            >
              APPLY HERE
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
