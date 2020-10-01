import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../utils/SectionProps'
import Button from '../elements/Button'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

const isCompany = false

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
      'hero section center-content cursor',
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
                {isCompany ? (
                  <p className="m-0 mb-32 reveal-from-top" data-reveal-delay="300">
                    We're proud to build the HOPR network for the HOPR Association.
                  </p>
                ) : (
                  <>
                    <p className="m-0 mb-32 reveal-from-top big-title pb-32" data-reveal-delay="300">
                      Changing Data Privacy For Good
                    </p>
                    {/* <div className="order_circle">
                        Order a HOPR Node PC
                      </div>*/}
                    <p className="m-0 mb-32 reveal-from-top" data-reveal-delay="350">
                      The HOPR protocol ensures everyone has control of their privacy, data, and identity.
                    </p>
                    <Button
                      color="primary"
                      tag="a"
                      href="https://saentis.hoprnet.org"
                      target="_blank"
                      className="reveal-from-top"
                      rel="noopener noreferrer"
                      data-reveal-delay="400"
                    >
                      Join Incentivized Testnet
                    </Button>
                  </>
                )}
              </div>
            </div>
            {/* <div className="hero-figure reveal-from-bottom" data-reveal-delay="600">
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
            </div> */}
          </div>
        </div>
      </section>
    )
  }
}

HeroFull.propTypes = propTypes
HeroFull.defaultProps = defaultProps

export default HeroFull
