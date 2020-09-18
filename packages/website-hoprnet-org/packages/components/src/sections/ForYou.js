import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import Image from '../elements/Image'
import { SectionProps } from '../utils/SectionProps'
import FooterSocial from '../layout/partials/FooterSocial'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const ForYou = props => {
  return (
    <GenericSection {...props}>
      <div className="center-content">
        <div className="container-sm">
          <p className="section-header mt-0 mb-0 reveal-from-top big-title" data-reveal-delay="150">
            Join The HOPR Community
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
            You can reach us on any of these channels:
            <br />
            <br />
            <FooterSocial className="large" invertColor />
          </div>
        </div>
      </div>
    </GenericSection>
  )
}

ForYou.propTypes = propTypes
ForYou.defaultProps = defaultProps

export default ForYou
