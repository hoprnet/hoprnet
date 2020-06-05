import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import { SectionProps } from '../../utils/SectionProps'

const propTypes = {
  children: PropTypes.node,
  ...SectionProps.types,
}

const defaultProps = {
  children: null,
  ...SectionProps.defaults,
}

class Blog extends React.Component {
  componentDidMount() {
    // add pixelpoint script
    let tracker = window.document.createElement('script')
    let firstScript = window.document.getElementsByTagName('script')[0]
    tracker.defer = true
    tracker.src = 'https://medium-widget.pixelpoint.io/widget.js'
    firstScript.parentNode.insertBefore(tracker, firstScript)

    tracker.onload = () => {
      // eslint-disable-next-line
      MediumWidget.Init({
        renderTo: '#medium-widget',
        params: {
          resource: 'https://medium.com/hoprnet',
          postsPerLine: 2,
          limit: 4,
          picture: 'big',
          fields: ['author', 'publishAt'],
          ratio: 'landscape',
        },
      })
    }
  }

  render() {
    return (
      <GenericSection {...this.props}>
        <div className="center-content">
          <div className="container-sm">
            <h2 className="section-header mt-0 mb-0 reveal-from-top" data-reveal-delay="150">
              Blog:
            </h2>
            <div className="reveal-from-top" data-reveal-delay="300">
              <div id="medium-widget" />
            </div>
          </div>
        </div>
      </GenericSection>
    )
  }
}

Blog.propTypes = propTypes
Blog.defaultProps = defaultProps

export default Blog
