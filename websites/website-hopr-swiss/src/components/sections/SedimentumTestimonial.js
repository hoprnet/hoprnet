import React from 'react'
import classNames from 'classnames'
import { SectionTilesProps } from '../utils/SectionProps'
import SectionHeader from './partials/SectionHeader'
import Image from '../elements/Image'

const propTypes = {
  ...SectionTilesProps.types,
}

const defaultProps = {
  ...SectionTilesProps.defaults,
}

class Testimonial extends React.Component {
  render() {
    const {
      className,
      topOuterDivider,
      bottomOuterDivider,
      topDivider,
      bottomDivider,
      hasBgColor,
      invertColor,
      pushLeft,
      ...props
    } = this.props

    const outerClasses = classNames(
      'testimonial section',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'testimonial-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const tilesClasses = classNames('tiles-wrap', pushLeft && 'push-left')

    return (
      <div className={outerClasses}>
        <div className={innerClasses}>
          <div className={tilesClasses}>
            <div className="reveal-from-bottom" data-reveal-container=".tiles-wrap">
              <div
                className="tiles-item-inner has-shadow"
                style={{
                  maxWidth: '900px',
                }}
              >
                <div className="testimonial-item-header mb-16">
                  <div className="testimonial-item-image">
                    <Image
                      src={require('@hoprnet/assets/images/partners/sedimentum_sandro.jpg')}
                      alt="Testimonial 01"
                      width={48}
                      height={48}
                    />
                  </div>
                </div>
                <div className="testimonial-item-content">
                  <p className="text-sm mb-0">
                    For Sedimentum data privacy is everything. Our customers are healthcare institutions and strict data
                    privacy requirements are necessary in that field. Sedimentum protects and supports life in a
                    privacy-preserving manner and the metadata protection of HOPR, who ensures that our customers' data
                    is being relayed from point-to-point securely and privately, is one crucial component to achieve
                    high failproof privacy guarantees. With HOPR we have found the right partner, who shares the same
                    values and vision with us.
                  </p>
                </div>
                <div className="testimonial-item-footer text-xs fw-500 mt-32 mb-0 has-top-divider">
                  <span className="testimonial-item-name">SANDRO CILURZO, CEO & Founder of Sedimentum</span>
                  {/* <span className="text-color-low"> - </span> */}
                  {/* <span className="testimonial-item-link">
                    <a href="#0">AppName</a>
                  </span> */}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    )
  }
}

Testimonial.propTypes = propTypes
Testimonial.defaultProps = defaultProps

export default Testimonial
