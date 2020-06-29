import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import { SectionProps } from '../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

const youtubeIds = ['mcnezYJXuXw', 'wH48dy6PjVg', 'YN8BEF1JIQ0', 'lHQBiZmCLBY', 'kZiCoR1DYSg']

const Videos = props => {
  return (
    <GenericSection {...props} id="videos" hasBgColor invertColor>
      <div className="center-content">
        <div className="container-sm">
          <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
            Videos
          </h2>
          <div className="reveal-from-top" data-reveal-delay="300">
            {youtubeIds.map(id => (
              <iframe
                key={id}
                title={id}
                width="400"
                height="225"
                src={`https://www.youtube-nocookie.com/embed/${id}`}
                frameBorder="0"
                allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture"
                allowFullScreen
              />
            ))}
          </div>
        </div>
      </div>
    </GenericSection>
  )
}

Videos.propTypes = propTypes
Videos.defaultProps = defaultProps

export default Videos
