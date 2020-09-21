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

const isCompany = true

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
      'features-tiles section center-content',
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
      title: 'HOPR is',
      paragraph: undefined,
    }

    const sectionHeader2 = {
      title: 'We want you to grow',
      paragraph: undefined,
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content" />
            <div className={tilesClasses}>
              <div className="tiles-item reveal-from-bottom" data-reveal-container=".tiles-wrap">
                <a href="/layer0-data-privacy#enabling_data_privacy">
                  <div className="tiles-item-inner">
                    <div className="features-tiles-item-header">
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/common-file-lock@140x140.png')}
                          alt="File Lock Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </div>
                    <div className="features-tiles-item-content">
                      <h4 className="mt-0 mb-8">Enabling Data Privacy</h4>
                      <p className="m-0 text-sm">
                        We're building the HOPR network to give people and companies control over their level of data
                        privacy.
                      </p>
                    </div>
                  </div>
                </a>
              </div>

              <div
                className="tiles-item reveal-from-bottom"
                data-reveal-container=".tiles-wrap"
                data-reveal-delay="100"
              >
                <a href="/layer0-data-privacy#open_source_support">
                  <div className="tiles-item-inner">
                    <div className="features-tiles-item-header">
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/user-signal@140x140.png')}
                          alt="Sharing Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </div>
                    <div className="features-tiles-item-content">
                      <h4 className="mt-0 mb-8">Open Source Support</h4>
                      <p className="m-0 text-sm">
                        Our experts are actively supporting the blockchain community to make this space a better place,
                        for the good of all of us.
                      </p>
                    </div>
                  </div>
                </a>
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
                    <a href="/layer0-data-privacy#community">
                      <div className="tiles-item-inner">
                        <div className="features-tiles-item-header">
                          <p className="mt-0 mb-24 text-sm">Personal</p>
                          <div className="features-tiles-item-image mb-16">
                            <Image
                              src={require('../assets/images/icons/love-heart-keyhole@140x140.png')}
                              alt="Heart Icon"
                              width={56}
                              height={56}
                            />
                          </div>
                        </div>
                        <div className="features-tiles-item-content">
                          <h4 className="mt-0 mb-8">Hackathons + Community</h4>
                          <p className="m-0 text-sm">
                            We're organizing events to grow our network and give back to the Community
                            <br />
                            <span className="text-color-high">(coming mid-July 2020)</span>.
                          </p>
                        </div>
                      </div>
                    </a>
                  </div>

                  <a
                    href={isCompany ? 'mailto:rik.krieger@hoprnet.org?subject=Partnership' : undefined}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <div
                      className="tiles-item reveal-from-bottom"
                      data-reveal-container=".tiles-wrap"
                      data-reveal-delay="300"
                    >
                      <div className="tiles-item-inner">
                        <div className="features-tiles-item-header">
                          <p className="mt-0 mb-24 text-sm">Professional</p>
                          <div className="features-tiles-item-image mb-16">
                            <Image
                              src={require('../assets/images/icons/building-modern@140x140.png')}
                              alt="Modern Building Icon"
                              width={56}
                              height={56}
                            />
                          </div>
                        </div>
                        <div className="features-tiles-item-content">
                          <h4 className="mt-0 mb-8">Your company</h4>
                          <p className="m-0 text-sm">
                            HOPR is looking for partners who want to benefit from our network. If data privacy is
                            important to you, get in touch.
                          </p>
                        </div>
                      </div>
                    </div>
                  </a>
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
