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

console.log(process.env)
const isCompany = true

class Clients extends React.Component {
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
      'clients section reveal-fade',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'clients-inner section-inner',
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
                title: 'Partners',
                paragraph: undefined,
              }}
              className="center-content header"
            />
            <ul className="list-reset">
              <li className="reveal-from-bottom">
                <a href="https://www.sedimentum.com/en/sedimentum-en/" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/sedimentum.png')}
                    alt="Sedimentum Logo"
                    className={imgClasses}
                    width={124}
                    height={24}
                  />
                </a>
              </li>
              {/* <li className="reveal-from-bottom" data-reveal-delay="150">
                <a href="https://www.bankfrick.li/en/" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/bank-frick.svg')}
                    alt="Bank Frick Logo"
                    className={imgClasses}
                    width={124}
                    height={24}
                  />
                </a>
              </li> */}
              {/* <li className="reveal-from-bottom" data-reveal-delay="150">
                <a href="https://www.hbl.ch/de/" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/hbl.png')}
                    alt="HBL Logo"
                    className={imgClasses}
                    width={124}
                    height={24}
                  />
                </a>
              </li> */}
              <li className="reveal-from-bottom" data-reveal-delay="150">
                <a href="https://www.froriep.com/de/" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/froriep.png')}
                    alt="Froriep Logo"
                    className={imgClasses}
                    width={124}
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
                  href={isCompany ? 'mailto:rik.krieger@hoprnet.org?subject=Partnership' : undefined}
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  Want to become our partner?
                </Button>
              </div>
            ) : null}
          </div>
        </div>
      </section>
    )
  }
}

Clients.propTypes = propTypes
Clients.defaultProps = defaultProps

export default Clients
