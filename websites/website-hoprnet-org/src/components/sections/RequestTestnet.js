import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
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

class RequestTestnet extends React.Component {
  render() {
    return (
      <GenericSection {...this.props}>
        <div className="token center-content">
          <div className="container-sm">
            <SectionHeader
              data={{
                title: 'HOPR testnet access:',
                paragraph: undefined,
              }}
            />
            <iframe
              title="Request Testnet Access"
              src="https://docs.google.com/forms/d/e/1FAIpQLScnOP3NqdL-5bmDKlmAfhhyx4DG7PZ7ji5UhRuMVhbhG1e6Cg/viewform?embedded=true&hl=en"
              width="700"
              height="650"
              frameBorder="0"
              marginHeight="0"
              marginWidth="0"
            >
              Loading…
            </iframe>
          </div>
        </div>
      </GenericSection>
    )
  }
}

RequestTestnet.propTypes = propTypes
RequestTestnet.defaultProps = defaultProps

export default RequestTestnet
