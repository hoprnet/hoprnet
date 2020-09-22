import React from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'

const propTypes = {
  data: PropTypes.shape({
    title: PropTypes.string,
    paragraph: PropTypes.oneOfType([PropTypes.string, PropTypes.node]),
  }).isRequired,
  children: PropTypes.node,
  tag: PropTypes.oneOf(['h1', 'h2', 'h3']),
}

const defaultProps = {
  children: null,
  tag: 'h2',
}

class SectionHeader extends React.Component {
  render() {
    const { className, data, children, tag, ...props } = this.props
    const classes = classNames('section-header', className)
    const Component = tag
    const paragraphIsComponent = typeof data.paragraph !== 'string'

    return (
      <React.Fragment>
        {(data.title || data.paragraph) && (
          <div {...props} className={classes}>
            <div className="container-sm">
              {children}
              {data.title && (
                <Component className={classNames('mt-0', data.paragraph ? 'mb-16' : 'mb-0')}>{data.title}</Component>
              )}
              {data.paragraph && (paragraphIsComponent ? data.paragraph : <p className="m-0">{data.paragraph}</p>)}
            </div>
          </div>
        )}
      </React.Fragment>
    )
  }
}

SectionHeader.propTypes = propTypes
SectionHeader.defaultProps = defaultProps

export default SectionHeader
