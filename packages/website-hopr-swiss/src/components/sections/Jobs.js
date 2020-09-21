import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import SectionHeader from './partials/SectionHeader'
import { SectionProps } from '../utils/SectionProps'
import insertScript from '../utils/insertScript'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const isCompany = true

class Jobs extends React.Component {
  componentDidMount() {
    // add jobbase iframe
    insertScript('https://hoprnet.jobbase.io/widget/iframe.js')
  }

  render() {
    return (
      <GenericSection {...this.props}>
        <div className="container-sm">
          <SectionHeader
            data={{
              title: 'Jobs',
              paragraph:
                this.props.forceIsCompany || isCompany
                  ? 'Want to join our team?'
                  : 'Companies building the HOPR ecosystem',
            }}
            className="center-content"
          />
          <div className="reveal-from-top" data-reveal-delay="300">
            <div id="psJobWidget" />
          </div>
        </div>
      </GenericSection>
    )
  }
}

Jobs.propTypes = propTypes
Jobs.defaultProps = defaultProps

export default Jobs
