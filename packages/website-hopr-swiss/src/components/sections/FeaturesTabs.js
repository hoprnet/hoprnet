import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../utils/SectionProps'
import SectionHeader from './partials/SectionHeader'
import Tabs, { TabList, Tab } from './../elements/Tabs'
import Image from '../elements/Image'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

const isCompany = true

class FeaturesTabs extends React.Component {
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
      'features-tabs section center-content',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'features-tabs-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const sectionHeader = {
      title: 'HOPR for',
      paragraph: undefined,
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content" />
            <Tabs active={!redirect ? 'tab-a' : undefined}>
              <TabList>
                <Tab tabId="tab-a" className={redirect ? 'is-active' : undefined}>
                  <a href="https://github.com/hoprnet" target="_blank" rel="noopener noreferrer">
                    <div className="features-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="50">
                      <Image src={require('../assets/images/icons/shield@140x140.png')} alt="Shield Icon" />
                    </div>
                    <div className="text-color-high text-sm">Privacy Experts</div>
                  </a>
                </Tab>
                <Tab tabId="tab-b" className={redirect ? 'is-active' : undefined}>
                  <a href="https://github.com/hoprnet" target="_blank" rel="noopener noreferrer">
                    <div className="features-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="100">
                      <Image src={require('../assets/images/icons/lock-4@140x140.png')} alt="Lock Icon" />
                    </div>
                    <div className="text-color-high text-sm">Cryptographers</div>
                  </a>
                </Tab>
                <Tab tabId="tab-c" className={redirect ? 'is-active' : undefined}>
                  <a href="http://docs.hoprnet.org/" target="_blank" rel="noopener noreferrer">
                    <div className="features-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="150">
                      <Image
                        src={require('../assets/images/icons/programming-team-chat-3@140x140.png')}
                        alt="Programming Team Icon"
                      />
                    </div>
                    <div className="text-color-high text-sm">Techies</div>
                  </a>
                </Tab>
                <Tab tabId="tab-d" className={redirect ? 'is-active' : undefined}>
                  <a
                    href={isCompany ? 'mailto:rik.krieger@hoprnet.org?subject=Partnership' : undefined}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <div className="features-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="200">
                      <Image
                        src={require('../assets/images/icons/light-bulb-shine@140x140.png')}
                        alt="Light Bulb Icon"
                      />
                    </div>
                    <div className="text-color-high text-sm">Entrepreneurs</div>
                  </a>
                </Tab>
                <Tab tabId="tab-e" className={redirect ? 'is-active' : undefined}>
                  <a href="http://docs.hoprnet.org/" target="_blank" rel="noopener noreferrer">
                    <div className="features-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="250">
                      <Image src={require('../assets/images/icons/outdoors-mining@140x140.png')} alt="Pickaxe Icon" />
                    </div>
                    <div className="text-color-high text-sm">Miners & Stakers</div>
                  </a>
                </Tab>
                <Tab tabId="tab-f" className={redirect ? 'is-active' : undefined}>
                  <a
                    href="mailto:sebastian.buergel@hoprnet.org?subject=Investment"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <div className="features-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="250">
                      <Image
                        src={require('../assets/images/icons/professions-man-office-1@140x140.png')}
                        alt="Investor Icon"
                      />
                    </div>
                    <div className="text-color-high text-sm">Investors</div>
                  </a>
                </Tab>
              </TabList>
            </Tabs>
          </div>
        </div>
      </section>
    )
  }
}

FeaturesTabs.propTypes = propTypes
FeaturesTabs.defaultProps = defaultProps

export default FeaturesTabs
