import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import SectionHeader from './partials/SectionHeader'
import { SectionProps } from '../../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

class Token extends React.Component {
  componentDidMount() {
    // add jobbase iframe
    let tracker = window.document.createElement('script')
    let firstScript = window.document.getElementsByTagName('script')[0]
    tracker.defer = true
    tracker.src = 'https://app.mailjet.com/statics/js/iframeResizer.min.js'
    firstScript.parentNode.insertBefore(tracker, firstScript)
  }

  render() {
    return (
      <GenericSection {...this.props}>
        <div className="token center-content">
          <div className="container-sm">
            <SectionHeader
              data={{
                title: 'HOPR Token:',
                paragraph: 'You want to know more about the sale of our Token, subscribe here:',
              }}
            />
            <div className="iframe-container">
              <iframe
                title="mailjet"
                src="https://app.mailjet.com/widget/iframe/5tV6/DH3"
                className="mj-w-res-iframe"
                scrolling="no"
                frameBorder="0"
                marginHeight="0"
                marginWidth="0"
              />
            </div>
          </div>
        </div>
      </GenericSection>
    )
  }
}

Token.propTypes = propTypes
Token.defaultProps = defaultProps

export default Token
