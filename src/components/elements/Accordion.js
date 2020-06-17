import React from 'react'
import classNames from 'classnames'

const Accordion = ({ className, ...props }) => {
  const classes = classNames('accordion list-reset mb-0', className)

  return <ul {...props} className={classes} />
}

export default Accordion
