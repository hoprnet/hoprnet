import React from 'react'
import classNames from 'classnames'
import { SectionTilesProps } from '../utils/SectionProps'
import Image from '../elements/Image'

const propTypes = {
  ...SectionTilesProps.types,
}

const defaultProps = {
  ...SectionTilesProps.defaults,
}

const isCompany = true

class TeamAndInvestors extends React.Component {
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
      'section center-content',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider',
      'teamAndInvestors-tiles-inner'
    )

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <div className="tiles-item reveal-from-top">
              <div className="tiles-item-content center-content-mobile">
                <a href="/who-is-HOPR#team">
                  <h3 className="mt-0 mb-0" data-reveal-container=".tiles-item">
                    Team
                  </h3>
                  <div
                    className="has-shadow has-bg-color invert-color card"
                    style={{
                      backgroundColor: isCompany ? '#5f0220' : '#53A3B9',
                    }}
                  >
                    <Image
                      className={isCompany ? 'img-to-white' : undefined}
                      src={
                        isCompany
                          ? require('../assets/images/icons/multiple-users-1@140x140.png')
                          : require('../assets/images/cards/team-card.png')
                      }
                    />
                  </div>
                </a>
              </div>
            </div>

            {isCompany ? null : (
              // <div className="tiles-item reveal-from-top">
              //   <div className="tiles-item-content center-content-mobile">
              //     <a href="/who-is-HOPR#investors">
              //       <h3 className="mt-0 mb-0" data-reveal-container=".tiles-item">
              //         Investors
              //       </h3>
              //       <div className="has-shadow has-bg-color invert-color card" style={{ backgroundColor: '#2E9AB9' }}>
              //         <Image src={require('../assets/images/cards/investors-card.png')} />
              //       </div>
              //     </a>
              //   </div>
              // </div>
              <div style={{ maxWidth: '500px', paddingTop: '55px' }}>
                HOPR is a digital commons built by and for an open community that has no single owner or single point of
                failure. At the same time delivery of complex software projects needs commitment and accountability.
                <br />
                Therefore the HOPR Association provided a first grant to the HOPR Services AG to develop and support a
                first version of the HOPR protocol in Typescript to bootstrap the ecosystem.
              </div>
            )}
          </div>
        </div>
      </section>
    )
  }
}

TeamAndInvestors.propTypes = propTypes
TeamAndInvestors.defaultProps = defaultProps

export default TeamAndInvestors
