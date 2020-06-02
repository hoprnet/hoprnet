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
      <div {...props} className="center-content">
        <div className="container-ms">
          <h2 className="mt-0 mb-0">Governance:</h2>
          Text by Roni & Sebastian deCEnGov
        </div>
      </div>
    </GenericSection>
  )
}

Governance.propTypes = propTypes
Governance.defaultProps = defaultProps

export default Governance
