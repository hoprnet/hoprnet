import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Substack from '../elements/Substack'
import SectionHeader from './partials/SectionHeader'
import { SectionProps } from '../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

class Token extends React.Component {
  render() {
    return (
      <GenericSection {...this.props}>
        <div className="token center-content">
          <div className="container-sm">
            <SectionHeader
              data={{
                title: 'HOPR Token',
                paragraph: 'Want to know more about our token sale? Subscribe here:',
              }}
            />
            <Substack />
          </div>
        </div>
      </GenericSection>
    )
  }
}

Token.propTypes = propTypes
Token.defaultProps = defaultProps

export default Token
