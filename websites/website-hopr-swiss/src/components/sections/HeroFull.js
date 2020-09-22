import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../utils/SectionProps'
import Image from '../elements/Image'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

const isCompany = true

class HeroFull extends React.Component {
  render() {
    const {
      className,
      topOuterDivider,
      bottomOuterDivider,
      topDivider,
      bottomDivider,
      hasBgColor,
      invertColor,
      ...props
    } = this.props

    const outerClasses = classNames(
      'hero section center-content',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'hero-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    return (
      <section {...props} className={outerClasses}>
        <div className="container-sm">
          <div className={innerClasses}>
            <div className="hero-content">
              {/* <h1 className="mt-0 mb-16 reveal-from-top" data-reveal-delay="150">
                {isCompany ? 'HOPR Services' : 'HOPR'}
              </h1> */}
              <div className="container-sm">
                <p className="m-0 mb-32 reveal-from-top" data-reveal-delay="300">
                  {isCompany ? (
                    "We're proud to build the HOPR network for the HOPR Association."
                  ) : (
                    <>
                      <h3>Changing Data Privacy For The Good</h3>
                      The HOPR protocol ensures everyone has control of their privacy, data, and identity.
                    </>
                  )}
                </p>
              </div>
            </div>
            <div className="hero-figure reveal-from-bottom" data-reveal-delay="600">
              <Image
                className="has-shadow"
                src={require('../assets/images/Layer0-Medical-Data-Privacy-Blockchain.png')}
                alt="image of layer0 data privacy blockchain solution for Web3"
                width={896}
                height={504}
                style={{
                  borderRadius: '15px',
                }}
              />
            </div>
          </div>
        </div>
      </section>
    )
  }
}

HeroFull.propTypes = propTypes
HeroFull.defaultProps = defaultProps

export default HeroFull
