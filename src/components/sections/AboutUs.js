import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Image from '../elements/Image'
import { SectionProps } from '../../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const AboutUs = props => {
  return (
    <GenericSection {...props}>
      <div className="center-content">
        <div className="container-sm">
          <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
            About us:
          </h2>
          <div className="mb-32 hero-figure reveal-from-top" data-reveal-delay="200">
            <Image
              className="has-shadow"
              src={require('../../assets/images/hopr-illustration-small.png')}
              alt="Hero"
              width={896}
              height={504}
              style={{
                borderRadius: '15px',
              }}
            />
          </div>
          <div className="pt-32 reveal-from-top" data-reveal-delay="300">
            We're a team of highly motivated experts with a single shared goal:
            <br />
            universal data privacy.
            <br />
            <br />
            We know that this is an ambitious and difficult target, and we won't be able to achieve it alone, but we
            believe HOPR can provide a significant and essential piece of the puzzle, ready for others to build on.
            <br />
            <br />
            Our vision is a world where privacy is available for anyone and everyone who wants it.
          </div>
        </div>
      </div>
    </GenericSection>
  )
}

AboutUs.propTypes = propTypes
AboutUs.defaultProps = defaultProps

export default AboutUs
