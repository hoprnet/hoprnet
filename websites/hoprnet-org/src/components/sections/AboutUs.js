import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Image from '../elements/Image'
import { SectionProps } from '../utils/SectionProps'

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
      <div className="center-content whole-page">
        <div className="container-sm">
          <p className="section-header mt-0 mb-0 reveal-from-top big-title" data-reveal-delay="150">
            Independent, Incorruptible, And Indestructible
          </p>
          {/* <div className="mb-32 hero-figure reveal-from-top" data-reveal-delay="200">
            <Image
              className="has-shadow"
              src={require('../assets/images/Web3-Data-Privacy.png')}
              alt="image of Web3 data privacy and protection"
              width={896}
              height={504}
              style={{
                borderRadius: '15px',
              }}
            />
          </div> */}
          <div className="pt-32 reveal-from-top" data-reveal-delay="300">
            We're a team of highly motivated experts with a single shared goal:
            <br />
            universal data privacy.
            <br />
            <br />
            With the HOPR protocol, companies and users can decide for themselves who can view their data and who can't.
            <br />
            <br />
            With the HOPR protocol, data security is solved, letting everyone focus on helping people through
            digitalization.
            <br />
            <br />
            The HOPR community is building the digital privacy landscape of tomorrow. We invite everybody to join our
            movement.
            <br />
            <br />
            Learn more about the team that buidls the HOPR network{' '}
            <a
              href="https://hopr.swiss/who-is-HOPR#team"
              target="_blank"
              rel="noopener noreferrer"
              className="underline"
            >
              here
            </a>
            .
          </div>
        </div>
      </div>
    </GenericSection>
  )
}

AboutUs.propTypes = propTypes
AboutUs.defaultProps = defaultProps

export default AboutUs
