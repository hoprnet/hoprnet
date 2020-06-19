import React from 'react'
import classNames from 'classnames'
import { SectionTilesProps } from '../../utils/SectionProps'
import Image from '../elements/Image'

const propTypes = {
  ...SectionTilesProps.types,
}

const defaultProps = {
  ...SectionTilesProps.defaults,
}

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
                <a href="/hopr/#team">
                  <h3 className="mt-0 mb-0" data-reveal-container=".tiles-item">
                    Team
                  </h3>
                  <div className="has-shadow has-bg-color invert-color card" style={{ backgroundColor: '#53A3B9' }}>
                    <Image src={require('../../assets/images/cards/team-card.png')} />
                  </div>
                </a>
              </div>
            </div>

            {/* <div className="tiles-item reveal-from-top">
              <div className="tiles-item-content center-content-mobile">
                <a href="/hopr/#investors">
                  <h3 className="mt-0 mb-0" data-reveal-container=".tiles-item">
                    Investors
                  </h3>
                  <div className="has-shadow has-bg-color invert-color card" style={{ backgroundColor: '#2E9AB9' }}>
                    <Image src={require('../../assets/images/cards/investors-card.png')} />
                  </div>
                </a>
              </div>
            </div> */}
          </div>
        </div>
      </section>
    )
  }
}

TeamAndInvestors.propTypes = propTypes
TeamAndInvestors.defaultProps = defaultProps

export default TeamAndInvestors
