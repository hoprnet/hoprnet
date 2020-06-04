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

const OpenSource = props => {
  return (
    <GenericSection {...props}>
      <div className="center-content">
        <div className="container-ms">
          <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
            Open Source Support:
          </h2>
          <div className="reveal-from-top" data-reveal-delay="300">
            WIP{' '}
            <span role="img" aria-label="">
              🚧
            </span>
          </div>
        </div>
      </div>
    </GenericSection>
  )
}

OpenSource.propTypes = propTypes
OpenSource.defaultProps = defaultProps

export default OpenSource
