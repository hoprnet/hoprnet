import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import { SectionProps } from '../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const Governance = props => {
  return (
    <GenericSection {...props}>
      <div className="governance center-content">
        <div className="container-sm">
          <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
            Governance
          </h2>
          <div className="container reveal-from-top" data-reveal-delay="300">
            <div className="column">
              At HOPR we're building the foundations for a more private and resilient web. But to succeed, we also need
              to challenge the economic power structures that shackle today's web to unhealthy and unsustainable
              business models.
              <br />
              <br />
              We need natively digital organizations to drive the next wave of inclusive innovation. Open technology
              platforms rely on and enable communities instead of emperors and shareholders.
            </div>
            <div className="column">
              The recent movement of Decentralized Autonomous Organizations (DAOs), provides participatory governance
              and economies for the blockchain era. However, DAOs cannot reach their full potential until they can
              co-exist with existing legal frameworks.
              <br />
              <br />
              HOPR is pioneering decentralized, community-enabling governance (DecenGov) as a techno-legal framework
              that combines the dynamics of communities with the efficiencies of crypto networks and the advantages of
              established legal bodies to govern collective efforts.
            </div>
          </div>
        </div>
      </div>
    </GenericSection>
  )
}

Governance.propTypes = propTypes
Governance.defaultProps = defaultProps

export default Governance
