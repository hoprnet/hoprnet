import React from 'react'
import classNames from 'classnames'
import { SectionTilesProps } from '../../utils/SectionProps'
import SectionHeader from './partials/SectionHeader'
import Image from '../elements/Image'

const propTypes = {
  ...SectionTilesProps.types,
}

const defaultProps = {
  ...SectionTilesProps.defaults,
}

class Team extends React.Component {
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
      'team section center-content',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'team-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const tilesClasses = classNames('tiles-wrap', pushLeft && 'push-left')

    const sectionHeader = {
      title: 'Team:',
      paragraph: undefined,
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content reveal-from-bottom" />
            <div className={tilesClasses}>
              <div className="tiles-item reveal-from-bottom" data-reveal-container=".tiles-wrap">
                <div className="tiles-item-inner">
                  <div className="team-item-header">
                    <a href="https://www.linkedin.com/in/scbuergel/" target="_blank" rel="noopener noreferrer">
                      <div className="team-item-image mb-24">
                        <Image
                          src={require('./../../assets/images/team/sebastian_burgel.jpg')}
                          alt="Team member 01"
                          width={180}
                          height={180}
                        />
                      </div>
                    </a>
                  </div>
                  <div className="team-item-content">
                    <h5 className="team-item-name mt-0 mb-4">Dr. Sebastian Bürgel</h5>
                    <div className="team-item-role text-xs fw-500 mb-8">Founder</div>
                    {/* <p className="m-0 text-sm"></p> */}
                  </div>
                </div>
              </div>

              <div
                className="tiles-item reveal-from-bottom"
                data-reveal-container=".tiles-wrap"
                data-reveal-delay="150"
              >
                <div className="tiles-item-inner">
                  <a
                    href="https://www.linkedin.com/in/robert-kiel-176878161/"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    <div className="team-item-header">
                      <div className="team-item-image mb-24">
                        <Image
                          src={require('./../../assets/images/team/robert_kiel.jpg')}
                          alt="Team member 02"
                          width={180}
                          height={180}
                        />
                      </div>
                    </div>
                  </a>
                  <div className="team-item-content">
                    <h5 className="team-item-name mt-0 mb-4">Robert Kiel</h5>
                    <div className="team-item-role text-xs fw-500 mb-8">Co-Founder, Crypto</div>
                    {/* <p className="m-0 text-sm"></p> */}
                  </div>
                </div>
              </div>

              <div
                className="tiles-item reveal-from-bottom"
                data-reveal-container=".tiles-wrap"
                data-reveal-delay="300"
              >
                <div className="tiles-item-inner">
                  <a href="https://www.linkedin.com/in/rikkrieger/" target="_blank" rel="noopener noreferrer">
                    <div className="team-item-header">
                      <div className="team-item-image mb-24">
                        <Image
                          src={require('./../../assets/images/team/rik_krieger.jpg')}
                          alt="Team member 03"
                          width={180}
                          height={180}
                        />
                      </div>
                    </div>
                  </a>
                  <div className="team-item-content">
                    <h5 className="team-item-name mt-0 mb-4">Rik Krieger, eMBA</h5>
                    <div className="team-item-role text-xs fw-500 mb-8">Co-Founder, Commercial</div>
                    {/* <p className="m-0 text-sm"></p> */}
                  </div>
                </div>
              </div>

              <div
                className="tiles-item reveal-from-bottom"
                data-reveal-container=".tiles-wrap"
                data-reveal-delay="450"
              >
                <div className="tiles-item-inner">
                  <a href="https://www.linkedin.com/in/jjperezaguinaga/" target="_blank" rel="noopener noreferrer">
                    <div className="team-item-header">
                      <div className="team-item-image mb-24">
                        <Image
                          src={require('./../../assets/images/team/jose_aguinaga.jpg')}
                          alt="Team member 04"
                          width={180}
                          height={180}
                        />
                      </div>
                    </div>
                  </a>
                  <div className="team-item-content">
                    <h5 className="team-item-name mt-0 mb-4">José Aguinaga</h5>
                    <div className="team-item-role text-xs fw-500 mb-8">Head of Engineering</div>
                    {/* <p className="m-0 text-sm"></p> */}
                  </div>
                </div>
              </div>

              <div
                className="tiles-item reveal-from-bottom"
                data-reveal-container=".tiles-wrap"
                data-reveal-delay="600"
              >
                <div className="tiles-item-inner">
                  <a href="https://www.linkedin.com/in/stevennonis/" target="_blank" rel="noopener noreferrer">
                    <div className="team-item-header">
                      <div className="team-item-image mb-24">
                        <Image
                          src={require('./../../assets/images/team/steven_nonis.jpg')}
                          alt="Team member 05"
                          width={180}
                          height={180}
                        />
                      </div>
                    </div>
                  </a>
                  <div className="team-item-content">
                    <h5 className="team-item-name mt-0 mb-4">Steven Nonis</h5>
                    <div className="team-item-role text-xs fw-500 mb-8">Software Engineer</div>
                    {/* <p className="m-0 text-sm"></p> */}
                  </div>
                </div>
              </div>

              <div
                className="tiles-item reveal-from-bottom"
                data-reveal-container=".tiles-wrap"
                data-reveal-delay="750"
              >
                <div className="tiles-item-inner">
                  <a href="https://www.linkedin.com/in/kevinleu/" target="_blank" rel="noopener noreferrer">
                    <div className="team-item-header">
                      <div className="team-item-image mb-24">
                        <Image
                          src={require('./../../assets/images/team/kevin-leu.jpg')}
                          alt="Team member 06"
                          width={180}
                          height={180}
                        />
                      </div>
                    </div>
                  </a>
                  <div className="team-item-content">
                    <h5 className="team-item-name mt-0 mb-4">Kevin Leu</h5>
                    <div className="team-item-role text-xs fw-500 mb-8">Head of Marketing</div>
                    <p className="m-0 text-sm"></p>
                  </div>
                </div>
              </div>

              <div
                className="tiles-item reveal-from-bottom"
                data-reveal-container=".tiles-wrap"
                data-reveal-delay="750"
              >
                <div className="tiles-item-inner">
                  <div className="team-item-header">
                    <div className="team-item-image mb-24">
                      <Image
                        src={require('./../../assets/images/team/rich_mcdowell.jpg')}
                        alt="Team member 06"
                        width={180}
                        height={180}
                      />
                    </div>
                  </div>
                  <div className="team-item-content">
                    <h5 className="team-item-name mt-0 mb-4">Rich McDowell</h5>
                    <div className="team-item-role text-xs fw-500 mb-8">Content Creator</div>
                    {/* <p className="m-0 text-sm"></p> */}
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

Team.propTypes = propTypes
Team.defaultProps = defaultProps

export default Team
