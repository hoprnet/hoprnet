import React from 'react'
import classNames from 'classnames'
import SectionHeader from '../sections/partials/SectionHeader'
import Image from '../elements/Image'
import Button from '../elements/Button'
import { SectionProps } from '../utils/SectionProps'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

class Investors extends React.Component {
  render() {
    const {
      className,
      topOuterDivider,
      bottomOuterDivider,
      topDivider,
      bottomDivider,
      hasBgColor,
      invertColor,
      showQuestion,
      ...props
    } = this.props

    const outerClasses = classNames(
      'clients section reveal-fade cursor',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'investors-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const imgClasses = classNames(invertColor ? 'img-to-white' : 'img-to-black')

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader
              data={{
                title: 'Investors:',
                paragraph: undefined,
              }}
              className="center-content header"
            />
            <ul className="list-reset">
              <li className="reveal-from-bottom">
                <a href="https://www.binance.com/en" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/binance.svg')}
                    alt="Binance Logo"
                    className={imgClasses}
                    width={124}
                    height={24}
                  />
                </a>
              </li>
              <li className="reveal-from-bottom" data-reveal-delay="150">
                <a href="https://www.sparkdigitalcapital.com/" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/sparklogo.png')}
                    alt="Spark Logo"
                    className={imgClasses}
                    width={124}
                    height={24}
                  />
                </a>
              </li>
              <li className="reveal-from-bottom" data-reveal-delay="150">
                <a href="https://twitter.com/fcslabs" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/focus_labs.png')}
                    alt="FocusLabs Logo"
                    className={imgClasses}
                    width={80}
                    height={24}
                  />
                </a>
              </li>
              <li className="reveal-from-bottom" data-reveal-delay="150">
                <a href="http://caballeroscapital.com/" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/caballeros_capital.png')}
                    alt="Caballeros Capital Logo"
                    className={imgClasses}
                    width={200}
                    height={24}
                  />
                </a>
              </li>
              {/* <li className="reveal-from-bottom" data-reveal-delay="150">
                <a href="https://www.bitcoinsuisse.com/" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/bitcoinsuisse.png')}
                    alt="Bitcoin Suisse Logo"
                    className={imgClasses}
                    width={124}
                    height={24}
                  />
                </a>
              </li> */}
            </ul>
            {showQuestion ? (
              <div className="question">
                <Button
                  color={invertColor ? 'secondary' : 'primary'}
                  tag="a"
                  href="mailto:sebastian.buergel@hoprnet.org?subject=Investment"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Want to become an investor?
                </Button>
              </div>
            ) : null}
          </div>
        </div>
      </section>
    )
  }
}

Investors.propTypes = propTypes
Investors.defaultProps = defaultProps

export default Investors
