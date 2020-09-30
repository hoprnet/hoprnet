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

const SetupHero = props => {
  return (
    <GenericSection id="notfound" {...props}>
      <div className="textContainer">
        Hey - we will have all setup instructions here as of Oct 1st.
        <br />
        <br />
        If you have any questions in the meantime, pls email us at{' '}
        <a href="mailto:First100@HOPRnet.org" target="_blank" rel="noopener noreferrer" className="underline">
          First100@HOPRnet.org
        </a>{' '}
        or checkout our{' '}
        <a href="https://t.me/hoprnet" target="_blank" rel="noopener noreferrer" className="underline">
          telegram
        </a>{' '}
        channels for more.
      </div>
      <div className="circleContainer">
        <div className="circle" />
      </div>
    </GenericSection>
  )
}

SetupHero.propTypes = propTypes
SetupHero.defaultProps = defaultProps

export default SetupHero
