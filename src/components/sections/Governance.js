import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import { SectionProps } from '../../utils/SectionProps'

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
      <div className="center-content">
        <div className="container-ms">
          <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
            Governance:
          </h2>
          <div className="reveal-from-top" data-reveal-delay="300">
            At HOPR we are building the foundations of a more private and resilient next web. But to do so, we also have
            to fundamentally challenge the economic power structures of today’s web.
            <br />
            <br />
            Governments and corporations have been major innovations in governing and organizing human progress over the
            past millennia. But today more than ever, we need natively digital organizations to drive the next wave of
            inclusive innovation.
            <br />
            <br />
            Open technology platforms rely on and enable communities instead of emperors and shareholders. The recent
            movement of Decentralized Autonomous Organizations (DAOs), provides participatory governance and economies
            for the blockchain era. However, DAOs can only unleash their full potential if they also benefit from
            established legal context.
            <br />
            <br />
            Therefore, at HOPR we pioneer decentralized, community-enabling governance - DeCEnGov - as a techno-legal
            framework that combines the dynamics of communities with the efficiencies of crypto networks and the
            advantages of established legal bodies to govern collective efforts.
            <br />
            <br />
            Link to FroRieb / DAO / DAA
          </div>
        </div>
      </div>
    </GenericSection>
  )
}

Governance.propTypes = propTypes
Governance.defaultProps = defaultProps

export default Governance
