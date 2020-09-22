import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../utils/SectionProps'
import SectionHeader from './partials/SectionHeader'
import Timeline from '../elements/Timeline'
import TimelineItem from '../elements/TimelineItem'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

class Roadmap extends React.Component {
  render() {
    const {
      className,
      topOuterDivider,
      bottomOuterDivider,
      topDivider,
      bottomDivider,
      hasBgColor,
      invertColor,
      ...props
    } = this.props

    const outerClasses = classNames(
      'roadmap section cursor',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'roadmap-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    const sectionHeader = {
      title: 'Product roadmap',
      paragraph:
        'Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.',
    }

    return (
      <section {...props} className={outerClasses}>
        <div className="container">
          <div className={innerClasses}>
            <SectionHeader data={sectionHeader} className="center-content" />
            <Timeline>
              <TimelineItem title="November 2019">
                Deployed a high-quality first release and conducted a market validation test
              </TimelineItem>
              <TimelineItem title="December 2019">
                Deployed a high-quality first release and conducted a market validation test
              </TimelineItem>
              <TimelineItem title="January 2020">
                Deployed a high-quality first release and conducted a market validation test
              </TimelineItem>
              <TimelineItem title="February 2020">
                Deployed a high-quality first release and conducted a market validation test
              </TimelineItem>
              <TimelineItem title="March 2020">
                Deployed a high-quality first release and conducted a market validation test
              </TimelineItem>
            </Timeline>
          </div>
        </div>
      </section>
    )
  }
}

Roadmap.propTypes = propTypes
Roadmap.defaultProps = defaultProps

export default Roadmap
