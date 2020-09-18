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
      'blogs-tabs section center-content',
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
      title: 'HOPR blogs',
      paragraph: undefined,
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content" />
            <Tabs active={!redirect ? 'tab-a' : undefined}>
              <a href="/do-business-with-HOPR#blog">
                <TabList>
                  <Tab tabId="tab-a" className={redirect ? 'is-active' : undefined}>
                    <div className="blogs-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="50">
                      <Image src={require('../assets/images/icons/shield@140x140.png')} alt="Shield Icon" />
                    </div>
                    <div className="text-color-high text-sm">Privacy Blog</div>
                  </Tab>
                  <Tab tabId="tab-b" className={redirect ? 'is-active' : undefined}>
                    <div className="blogs-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="100">
                      <Image src={require('../assets/images/icons/lock-4@140x140.png')} alt="Lock Icon" />
                    </div>
                    <div className="text-color-high text-sm">Crypto Blog</div>
                  </Tab>
                  <Tab tabId="tab-c" className={redirect ? 'is-active' : undefined}>
                    <div className="blogs-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="150">
                      <Image
                        src={require('../assets/images/icons/programming-team-chat-3@140x140.png')}
                        alt="Programming Team Icon"
                      />
                    </div>
                    <div className="text-color-high text-sm">Tech Blog</div>
                  </Tab>
                  <Tab tabId="tab-d" className={redirect ? 'is-active' : undefined}>
                    <div className="blogs-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="200">
                      <Image
                        src={require('../assets/images/icons/light-bulb-shine@140x140.png')}
                        alt="Light Bulb Icon"
                      />
                    </div>
                    <div className="text-color-high text-sm">News Blog</div>
                  </Tab>
                  {/* <Tab tabId="tab-e" className={redirect ? 'is-active' : undefined}>
                  <div className="blogs-tabs-tab-image mb-12 reveal-fade" data-reveal-offset="250">
                    <Image
                      src={require('../assets/images/icons/space-rocket-launch@140x140.png')}
                      alt="Pickaxe Icon"
                    />
                  </div>
                  <div className="text-color-high text-sm">Launchpad Blog</div>
                </Tab> */}
                </TabList>
              </a>
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
