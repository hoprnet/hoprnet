import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../../utils/SectionProps'
import SectionHeader from './partials/SectionHeader'
import Tabs, { TabList, Tab, TabPanel } from './../elements/Tabs'
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
            <Tabs active="tab-a">
              <TabList>
                <Tab tabId="tab-a">
                  <div className="features-tabs-tab-image mb-12">
                    <Image
                      src={require('./../../assets/images/features-tabs-icon-01.svg')}
                      alt="Tab icon 01"
                      width={56}
                      height={56}
                    />
                  </div>
                  <div className="text-color-high text-sm">Privacy Experts</div>
                </Tab>
                <Tab tabId="tab-b">
                  <div className="features-tabs-tab-image mb-12">
                    <Image
                      src={require('./../../assets/images/features-tabs-icon-02.svg')}
                      alt="Tab icon 02"
                      width={56}
                      height={56}
                    />
                  </div>
                  <div className="text-color-high text-sm">Cryptographers</div>
                </Tab>
                <Tab tabId="tab-c">
                  <div className="features-tabs-tab-image mb-12">
                    <Image
                      src={require('./../../assets/images/features-tabs-icon-03.svg')}
                      alt="Tab icon 03"
                      width={56}
                      height={56}
                    />
                  </div>
                  <div className="text-color-high text-sm">Techies</div>
                </Tab>
                <Tab tabId="tab-d">
                  <div className="features-tabs-tab-image mb-12">
                    <Image
                      src={require('./../../assets/images/features-tabs-icon-04.svg')}
                      alt="Tab icon 04"
                      width={56}
                      height={56}
                    />
                  </div>
                  <div className="text-color-high text-sm">Entrepreneurs</div>
                </Tab>
                <Tab tabId="tab-e">
                  <div className="features-tabs-tab-image mb-12">
                    <Image
                      src={require('./../../assets/images/features-tabs-icon-04.svg')}
                      alt="Tab icon 05"
                      width={56}
                      height={56}
                    />
                  </div>
                  <div className="text-color-high text-sm">Miners & Relayers</div>
                </Tab>
              </TabList>
              <TabPanel id="tab-a">
                <Image
                  className="has-shadow"
                  src={require('./../../assets/images/features-tabs-image.png')}
                  alt="Features tabs image 01"
                  width={896}
                  height={504}
                />
              </TabPanel>
              <TabPanel id="tab-b">
                <Image
                  className="has-shadow"
                  src={require('./../../assets/images/features-tabs-image.png')}
                  alt="Features tabs image 02"
                  width={896}
                  height={504}
                />
              </TabPanel>
              <TabPanel id="tab-c">
                <Image
                  className="has-shadow"
                  src={require('./../../assets/images/features-tabs-image.png')}
                  alt="Features tabs image 03"
                  width={896}
                  height={504}
                />
              </TabPanel>
              <TabPanel id="tab-d">
                <Image
                  className="has-shadow"
                  src={require('./../../assets/images/features-tabs-image.png')}
                  alt="Features tabs image 04"
                  width={896}
                  height={504}
                />
              </TabPanel>
              <TabPanel id="tab-e">
                <Image
                  className="has-shadow"
                  src={require('./../../assets/images/features-tabs-image.png')}
                  alt="Features tabs image 05"
                  width={896}
                  height={504}
                />
              </TabPanel>
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
