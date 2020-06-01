import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../../utils/SectionProps'
import Image from '../elements/Image'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

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

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <ul className="list-reset">
              <li className="reveal-from-bottom">
                <Image
                  src={require('./../../assets/images/client-binance.svg')}
                  alt="Binance Logo"
                  width={124}
                  height={24}
                />
              </li>
              <li className="reveal-from-bottom" data-reveal-delay="150">
                <Image
                  src={require('./../../assets/images/client-bank-frick.svg')}
                  alt="Bank Frick Logo"
                  width={124}
                  height={24}
                />
              </li>
              <li className="reveal-from-bottom" data-reveal-delay="300">
                <span>Become our partner</span>
              </li>
            </ul>
          </div>
        </div>
      </section>
    )
  }
}

Clients.propTypes = propTypes
Clients.defaultProps = defaultProps

export default Clients
