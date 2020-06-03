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

class Jobs extends React.Component {
  componentDidMount() {
    // add jobbase iframe
    let tracker = window.document.createElement('script')
    let firstScript = window.document.getElementsByTagName('script')[0]
    tracker.defer = true
    tracker.src = 'https://hoprnet.jobbase.io/widget/iframe.js'
    firstScript.parentNode.insertBefore(tracker, firstScript)
  }

  render() {
    return (
      <GenericSection {...this.props}>
        <div className="center-content">
          <div className="container-ms">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Jobs:
            </h2>
            <div className="reveal-from-top" data-reveal-delay="300">
              <div id="psJobWidget" />
            </div>
          </div>
        </div>
      </GenericSection>
    )
  }
}

Jobs.propTypes = propTypes
Jobs.defaultProps = defaultProps

export default Jobs
