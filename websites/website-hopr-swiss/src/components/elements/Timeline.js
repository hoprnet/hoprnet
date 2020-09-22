import React from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'

const propTypes = {
  children: PropTypes.node,
}

const defaultProps = {
  children: null,
}

const Timeline = ({ className, children, ...props }) => {
  const classes = classNames('timeline', className)

  return (
    <div {...props} className={classes}>
      <div className="timeline-wrap">{children}</div>
    </div>
  )
}

Timeline.propTypes = propTypes
Timeline.defaultProps = defaultProps

export default Timeline
