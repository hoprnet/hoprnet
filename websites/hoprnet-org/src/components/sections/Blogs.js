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

class Blogs extends React.Component {
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
      'blogs-tabs section center-content cursor',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'blogs-tabs-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const sectionHeader = {
      title: 'HOPR Blogs',
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
                  <a href="/do-business-with-HOPR#blog">
                    <div className="blogs-tabs-tab-image-reversed mb-12 reveal-fade" data-reveal-offset="50">
                      <Image src={require('../assets/images/icons/with-yellow-ball/shield.png')} alt="Shield Icon" />
                    </div>
                    <div className="text-color-default text-sm">Privacy Blog</div>
                  </a>
                </Tab>
                <Tab tabId="tab-b" className={redirect ? 'is-active' : undefined}>
                  <a href="/do-business-with-HOPR#blog">
                    <div className="blogs-tabs-tab-image-reversed mb-12 reveal-fade" data-reveal-offset="100">
                      <Image src={require('../assets/images/icons/with-yellow-ball/lock-4.png')} alt="Lock Icon" />
                    </div>
                    <div className="text-color-default text-sm">Crypto Blog</div>
                  </a>
                </Tab>
                <Tab tabId="tab-c" className={redirect ? 'is-active' : undefined}>
                  <a href="/do-business-with-HOPR#blog">
                    <div className="blogs-tabs-tab-image-reversed mb-12 reveal-fade" data-reveal-offset="150">
                      <Image
                        src={require('../assets/images/icons/with-yellow-ball/programming-team-chat-3.png')}
                        alt="Programming Team Icon"
                      />
                    </div>
                    <div className="text-color-default text-sm">Tech Blog</div>
                  </a>
                </Tab>
                <Tab tabId="tab-d" className={redirect ? 'is-active' : undefined}>
                  <a href="/do-business-with-HOPR#blog">
                    <div className="blogs-tabs-tab-image-reversed mb-12 reveal-fade" data-reveal-offset="200">
                      <Image
                        src={require('../assets/images/icons/with-yellow-ball/light-bulb-shine.png')}
                        alt="Light Bulb Icon"
                      />
                    </div>
                    <div className="text-color-default text-sm">News Blog</div>
                  </a>
                </Tab>
                {/* <Tab tabId="tab-e" className={redirect ? 'is-active' : undefined}>
                  <div className="blogs-tabs-tab-image-reversed mb-12 reveal-fade" data-reveal-offset="250">
                    <Image
                      src={require('../assets/images/icons/with-yellow-ball/space-rocket-launch.png')}
                      alt="Pickaxe Icon"
                    />
                  </div>
                  <div className="text-color-default text-sm">Launchpad Blog</div>
                </Tab> */}
              </TabList>
            </Tabs>
          </div>
        </div>
      </section>
    )
  }
}

Blogs.propTypes = propTypes
Blogs.defaultProps = defaultProps

export default Blogs
