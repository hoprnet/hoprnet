import React from 'react'
import classNames from 'classnames'
import { SectionTilesProps } from '../../utils/SectionProps'

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
            <div className="tiles-item has-shadow">
              <div className="tiles-item-content center-content-mobile">
                <a href="/HOPR/#team">
                  <h3 className="mt-0 mb-16 reveal-from-bottom" data-reveal-container=".tiles-item">
                    The people behind HOPR
                  </h3>
                </a>
                <p className="m-0 reveal-from-bottom" data-reveal-delay="100" data-reveal-container=".tiles-item">
                  Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et
                  dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
                  aliquip ex ea commodo consequat.
                </p>
              </div>
            </div>

            <div className="tiles-item has-shadow">
              <div className="tiles-item-content center-content-mobile">
                <a href="/HOPR/#investors">
                  <h3 className="mt-0 mb-16 reveal-from-bottom" data-reveal-container=".tiles-item">
                    The investors behind HOPR
                  </h3>
                </a>
                <p className="m-0 reveal-from-bottom" data-reveal-delay="100" data-reveal-container=".tiles-item">
                  Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et
                  dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut
                  aliquip ex ea commodo consequat.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>
    )
  }
}

TeamAndInvestors.propTypes = propTypes
TeamAndInvestors.defaultProps = defaultProps

export default TeamAndInvestors
