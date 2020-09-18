import React from 'react'
import PropTypes from 'prop-types'
import GenericSection from './GenericSection'
import { SectionProps } from '../utils/SectionProps'
import insertScript from '../utils/insertScript'

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
    const script = insertScript('https://medium-widget.pixelpoint.io/widget.js')

    script.onload = () => {
      // eslint-disable-next-line
      MediumWidget.Init({
        renderTo: '#medium-widget',
        params: {
          resource: 'https://medium.com/hoprnet',
          postsPerLine: 2,
          limit: undefined,
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
              Blog
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
