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

const isCompany = false

class FeaturesTiles extends React.Component {
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
      redirect,
      ...props
    } = this.props

    const outerClasses = classNames(
      'features-tiles section center-content cursor',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'features-tiles-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const tilesClasses = classNames('tiles-wrap', pushLeft && 'push-left')

    const sectionHeader = {
      title: 'HOPR Is',
      paragraph: '<p>HOPR is a protocol that relies on economic incentives on top of a blockchain. To incentivize people to run nodes supporting the network, we rely on token economics to reward users for powering the p2p network that the protocol creates</p>',
    }

    const sectionHeader2 = {
      title: 'We Want You To Grow',
      paragraph: undefined,
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content" />
            <div className={tilesClasses}>
              <div className="tiles-item reveal-from-bottom" data-reveal-container=".tiles-wrap">
                <div className="tiles-item-inner">
                  <div className="features-tiles-item-header">
                    <a href="/layer0-data-privacy#enabling_data_privacy">
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/with-blue-stroke/common-file-lock.png')}
                          alt="File Lock Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </a>
                  </div>
                  <div className="features-tiles-item-content">
                    <h4 className="mt-0 mb-8">Enabling Data Privacy</h4>
                    <p className="m-0 text-sm">
                      We're building the HOPR network to give people and companies control over their level of data
                      privacy.
                    </p>
                  </div>
                </div>
              </div>

              <div
                className="tiles-item reveal-from-bottom"
                data-reveal-container=".tiles-wrap"
                data-reveal-delay="100"
              >
                <div className="tiles-item-inner">
                  <div className="features-tiles-item-header">
                    <a href="/layer0-data-privacy#open_source_support">
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/with-blue-stroke/user-signal.png')}
                          alt="Sharing Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </a>
                  </div>
                  <div className="features-tiles-item-content">
                    <h4 className="mt-0 mb-8">Open Source Support</h4>
                    <p className="m-0 text-sm">
                      The HOPR community fights for the digital privacy of the future. Everybody can become a part of
                      our movement.
                    </p>
                  </div>
                </div>
              </div>
            </div>

            {!isCompany && (
              <>
                <SectionHeader data={sectionHeader2} className="center-content" />
                <div className={tilesClasses}>
                  <div
                    className="tiles-item reveal-from-bottom"
                    data-reveal-container=".tiles-wrap"
                    data-reveal-delay="200"
                  >
                    <div className="tiles-item-inner">
                      <a href="/layer0-data-privacy#community">
                        <div className="features-tiles-item-header">
                          <p className="mt-0 mb-24 text-sm">Personal</p>
                          <div className="features-tiles-item-image mb-16">
                            <Image
                              src={require('../assets/images/icons/with-blue-stroke/love-heart-keyhole.png')}
                              alt="Heart Icon"
                              width={56}
                              height={56}
                            />
                          </div>
                        </div>
                      </a>
                      <div className="features-tiles-item-content">
                        <h4 className="mt-0 mb-8">Hackathons + Community</h4>
                        <p className="m-0 text-sm">
                          From testnet game sessions to reward bounties, join our{' '}
                          <a
                            href="https://t.me/hoprnet"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-color-high underline"
                          >
                            Telegram
                          </a>{' '}
                          to become part of our growing community.
                          <br />
                        </p>
                      </div>
                    </div>
                  </div>

                  <div
                    className="tiles-item reveal-from-bottom"
                    data-reveal-container=".tiles-wrap"
                    data-reveal-delay="300"
                  >
                    <div className="tiles-item-inner">
                      <div className="features-tiles-item-header">
                        <a
                          href={isCompany ? 'mailto:rik.krieger@hoprnet.org?subject=Partnership' : undefined}
                          target="_blank"
                          rel="noopener noreferrer"
                        >
                          <p className="mt-0 mb-24 text-sm">Professional</p>
                          <div className="features-tiles-item-image mb-16">
                            <Image
                              src={require('../assets/images/icons/with-blue-stroke/building-modern.png')}
                              alt="Modern Building Icon"
                              width={56}
                              height={56}
                            />
                          </div>
                        </a>
                      </div>
                      <div className="features-tiles-item-content">
                        <h4 className="mt-0 mb-8">Your company</h4>
                        <p className="m-0 text-sm">
                          HOPR is providing worry-free privacy to everyone.{' '}
                          <a
                            href="/partners"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-color-high underline"
                          >
                            Reach out to us
                          </a>{' '}
                          if you and your customers deserve the best.
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </>
            )}
          </div>
        </div>
      </section>
    )
  }
}

FeaturesTiles.propTypes = propTypes
FeaturesTiles.defaultProps = defaultProps

export default FeaturesTiles
