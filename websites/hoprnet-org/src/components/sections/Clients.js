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

const isCompany = false

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
      'clients section reveal-fade cursor',
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

    const imgClasses = classNames(invertColor ? 'img-to-grey' : 'img-to-black')

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
              <li className="reveal-from-top" data-reveal-delay="150">
                <a
                  href="https://medium.com/hoprnet/attention-hopr-node-runners-3ce80ddf67c2"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Image
                    src={require('@hoprnet/assets/images/partners/avado.svg')}
                    alt="Avado Logo"
                    className={imgClasses}
                    height={24}
                    width={124}
                  />
                </a>
              </li>
              <li className="reveal-from-top" data-reveal-delay="150">
                <a
                  href="https://www.mondaq.com/shareholders/968208/the-importance-of-governance-in-building-ecosystems"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Image
                    src={require('@hoprnet/assets/images/partners/froriep.png')}
                    alt="Froriep Logo"
                    className={imgClasses}
                    height={24}
                    width={124}
                  />
                </a>
              </li>
              <li className="reveal-from-top" data-reveal-delay="150">
                <a
                  href="https://elrond.com/blog/enhanced-privacy-hopr-tools-network/"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Image
                    src={require('@hoprnet/assets/images/partners/elrond.png')}
                    alt="Elrond Logo"
                    className={imgClasses}
                    height={24}
                    width={124}
                  />
                </a>
              </li>
              <li className="reveal-from-top">
                <a
                  href="https://medium.com/hoprnet/privacy-as-dignity-2fcdc4bd14f7"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Image
                    src={require('@hoprnet/assets/images/partners/sedimentum.png')}
                    alt="Sedimentum Logo"
                    className={imgClasses}
                    height={24}
                    width={124}
                  />
                </a>
              </li>
              <li className="reveal-from-top" data-reveal-delay="150">
                <a href="https://www.swiss-medtech.ch/en" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/swiss_medtech.png')}
                    alt="Swiss Medtech Logo"
                    className={imgClasses}
                    height={24}
                    width={124}
                  />
                </a>
              </li>
              <li className="reveal-from-top" data-reveal-delay="150">
                <a
                  href="https://medium.com/hoprnet/introducing-our-new-incentivized-testnet-hopr-s%C3%A4ntis-612a05b36932"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Image
                    src={require('@hoprnet/assets/images/partners/xdai.png')}
                    alt="xDAI Logo"
                    className={imgClasses}
                    height={24 * 0.5}
                    width={124 * 0.5}
                  />
                </a>
              </li>
              <li className="reveal-from-top" data-reveal-delay="150">
                <a href="https://en.healthtech.ch/" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/health_tech_cluster.png')}
                    alt="Health Tech Cluster Logo"
                    className={imgClasses}
                    height={24}
                    width={124}
                  />
                </a>
              </li>
              <li className="reveal-from-top" data-reveal-delay="150">
                <a href="https://www.swiss-medtech.ch/en" target="_blank" rel="noopener noreferrer">
                  <Image
                    src={require('@hoprnet/assets/images/partners/swiss_healthcare_startups.png')}
                    alt="Swiss Healthcare Startups Logo"
                    className={imgClasses}
                    height={24}
                    width={124}
                  />
                </a>
              </li>
            </ul>
            {showQuestion ? (
              <div className="question">
                <Button
                  color={invertColor ? 'secondary' : 'primary'}
                  tag="a"
                  href={isCompany ? 'mailto:rik.krieger@hoprnet.org?subject=Partnership' : '/partners'}
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
