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

const sectionHeader = {
  title: 'HOPR Services',
  paragraph: undefined,
}

class Products extends React.Component {
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
      'products section center-content cursor',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'products-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const tilesClasses = classNames('tiles-wrap', pushLeft && 'push-left')

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} />
            <a href="mailto:partners@hopr.swiss" target="_blank" rel="noopener noreferrer">
              <div className={tilesClasses}>
                <div className="tiles-item reveal-from-bottom" data-reveal-container=".tiles-wrap">
                  <div className="tiles-item-inner">
                    <div className="features-tiles-item-header">
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/with-blue-stroke/hierarchy-8.png')}
                          alt="File Lock Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </div>
                    <div className="features-tiles-item-content">
                      <h4 className="mt-0 mb-24">Consulting for Web3 networks</h4>
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
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/with-blue-stroke/cloud-data-transfer.png')}
                          alt="Sharing Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </div>
                    <div className="features-tiles-item-content">
                      <h4 className="mt-0 mb-24">Building HOPR network with our partners</h4>
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
                      <div className="features-tiles-item-image mb-16">
                        <Image
                          src={require('../assets/images/icons/with-blue-stroke/iris-scan-lock.png')}
                          alt="Sharing Icon"
                          width={56}
                          height={56}
                        />
                      </div>
                    </div>
                    <div className="features-tiles-item-content">
                      <h4 className="mt-0 mb-24">Data privacy consulting for corporations</h4>
                    </div>
                  </div>
                </div>
              </div>
            </a>
          </div>
        </div>
      </section>
    )
  }
}

Products.propTypes = propTypes
Products.defaultProps = defaultProps

export default Products
