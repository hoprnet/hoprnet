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

class News extends React.Component {
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
      'news section',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'news-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const tilesClasses = classNames('tiles-wrap', pushLeft && 'push-left')

    const sectionHeader = {
      title: 'In the News',
      paragraph: undefined,
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content reveal-from-bottom" />
            <div className={tilesClasses}>
              <div className="tiles-item reveal-from-bottom">
                <div className="tiles-item-inner has-shadow">
                  <div className="news-item-image m-0 mb-12">
                    <a
                      href="https://www.coindesk.com/binance-labs-leads-1m-seed-round-in-crypto-tor-alternative-hopr"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <Image
                        src={require('@hoprnet/assets/images/partners/coindesk.png')}
                        alt="Binance Labs Logo"
                        width={344}
                        height={194}
                      />
                    </a>
                  </div>
                  <div className="news-item-content">
                    <div className="news-item-body">
                      {/* <h3 className="news-item-title h4 mt-0 mb-8">
                        <a href="https://labs.binance.com/">How to build anything</a>
                      </h3> */}
                      <p className="mb-16 text-sm quote">
                        Binance Labs, the incubation and seed funding arm of the world’s largest crypto exchange, has
                        made its first investment this year, backing decentralized privacy startup HOPR.
                      </p>
                    </div>
                    <div className="news-item-more text-xs mb-8">
                      <a
                        href="https://www.coindesk.com/binance-labs-leads-1m-seed-round-in-crypto-tor-alternative-hopr"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="underline"
                      >
                        Read more
                      </a>
                    </div>
                  </div>
                </div>
              </div>

              <div className="tiles-item reveal-from-bottom" data-reveal-delay="200">
                <div className="tiles-item-inner has-shadow">
                  <div className="news-item-image m-0 mb-12">
                    <a
                      href="https://cointelegraph.com/news/hopr-data-privacy-testnet-to-launch-following-investment-by-binance"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <Image
                        src={require('@hoprnet/assets/images/partners/cointelegraph.png')}
                        alt="Cointelegraph Logo"
                        width={344}
                        height={194}
                      />
                    </a>
                  </div>
                  <div className="news-item-content">
                    <div className="news-item-body">
                      {/* <h3 className="news-item-title h4 mt-0 mb-8">
                        <a href="https://cruip.com">How to build anything</a>
                      </h3> */}
                      <p className="mb-16 text-sm quote">
                        The privacy-focused network allows users to run HOPR nodes, stake and get rewards with HOPR
                        tokens while providing privacy for Web 3.
                      </p>
                    </div>
                    <div className="news-item-more text-xs mb-8">
                      <a
                        href="https://cointelegraph.com/news/hopr-data-privacy-testnet-to-launch-following-investment-by-binance"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="underline"
                      >
                        Read more
                      </a>
                    </div>
                  </div>
                </div>
              </div>

              <div className="tiles-item reveal-from-bottom" data-reveal-delay="400">
                <div className="tiles-item-inner has-shadow">
                  <div className="news-item-image m-0 mb-12">
                    <a
                      href="https://www.moneytoday.ch/news/das-schweizer-tech-startup-hopr-sammelt-1-million-dollar-ein-binance-labs-fuehrt-die-investitionsru/"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <Image
                        src={require('@hoprnet/assets/images/partners/moneytoday.png')}
                        alt="Moneytoday.ch Logo"
                        width={344}
                        height={194}
                      />
                    </a>
                  </div>
                  <div className="news-item-content">
                    <div className="news-item-body">
                      {/* <h3 className="news-item-title h4 mt-0 mb-8">
                        <a href="https://cruip.com">How to build anything</a>
                      </h3> */}
                      <p className="mb-16 text-sm quote">
                        From award-winning blockchain hackers, Silicon Valley tech veterans and a bold vision that could
                        become a reality in Switzerland.
                      </p>
                    </div>
                    <div className="news-item-more text-xs mb-8">
                      <a
                        href="https://www.moneytoday.ch/news/das-schweizer-tech-startup-hopr-sammelt-1-million-dollar-ein-binance-labs-fuehrt-die-investitionsru/"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="underline"
                      >
                        Read more
                      </a>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    )
  }
}

News.propTypes = propTypes
News.defaultProps = defaultProps

export default News
