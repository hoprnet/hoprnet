import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../../utils/SectionProps'
import SectionHeader from './partials/SectionHeader'
import Tabs, { TabList, Tab } from './../elements/Tabs'
import Image from '../elements/Image'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

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
      title: 'HOPR is built for',
      paragraph:
        'Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum — semper quis lectus nulla at volutpat diam ut venenatis.',
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content" />
            <Tabs>
              <TabList>
                <Tab tabId="tab-a">
                  <div className="features-tabs-tab-image mb-12">
                    <Image src={require('../../assets/images/icons/shield@140x140.png')} alt="Shield Icon" />
                  </div>
                  <div className="text-color-high text-sm">Privacy Experts</div>
                </Tab>
                <Tab tabId="tab-b">
                  <div className="features-tabs-tab-image mb-12">
                    <Image src={require('../../assets/images/icons/lock-4@140x140.png')} alt="Lock Icon" />
                  </div>
                  <div className="text-color-high text-sm">Cryptographers</div>
                </Tab>
                <Tab tabId="tab-c">
                  <div className="features-tabs-tab-image mb-12">
                    <Image
                      src={require('../../assets/images/icons/programming-team-chat-3@140x140.png')}
                      alt="Programming Team Icon"
                    />
                  </div>
                  <div className="text-color-high text-sm">Techies</div>
                </Tab>
                <Tab tabId="tab-d">
                  <div className="features-tabs-tab-image mb-12">
                    <Image
                      src={require('../../assets/images/icons/light-bulb-shine@140x140.png')}
                      alt="Light Bulb Icon"
                    />
                  </div>
                  <div className="text-color-high text-sm">Entrepreneurs</div>
                </Tab>
                <Tab tabId="tab-e">
                  <div className="features-tabs-tab-image mb-12">
                    <Image src={require('../../assets/images/icons/outdoors-mining@140x140.png')} alt="Pickaxe Icon" />
                  </div>
                  <div className="text-color-high text-sm">Miners & Relayers</div>
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
