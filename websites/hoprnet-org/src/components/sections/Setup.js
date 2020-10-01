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
        Hello there!
        <br />
        <br />
        Click{' '}
        <a
          href={require(`../assets/documents/HOPR_Quickstart_Manual.pdf`)}
          target="_blank"
          rel="noopener noreferrer"
          className="underline"
        >
          here
        </a>{' '}
        to download the HOPR Quickstart Manual.
        <br />
        <br />
        If you have any questions, feel free to email us at{' '}
        <a href="mailto:First100@HOPRnet.org" target="_blank" rel="noopener noreferrer" className="underline">
          First100@HOPRnet.org
        </a>{' '}
        or message us in our{' '}
        <a href="https://t.me/hoprnet" target="_blank" rel="noopener noreferrer" className="underline">
          community
        </a>{' '}
        telegram channel.
        <br />
        <br />
        New features and updates will be announced over our{' '}
        <a href="https://t.me/HOPRannouncements" target="_blank" rel="noopener noreferrer" className="underline">
          announcements
        </a>{' '}
        telegram channel.
      </div>
    </GenericSection>
  )
}

SetupHero.propTypes = propTypes
SetupHero.defaultProps = defaultProps

export default SetupHero
