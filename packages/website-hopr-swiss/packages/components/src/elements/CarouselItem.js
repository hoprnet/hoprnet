import React from 'react'
import classNames from 'classnames'

const CarouselItem = ({ className, ...props }) => {
  const classes = classNames('carousel-item', className)

  return <div {...props} className={classes} />
}

export default CarouselItem
