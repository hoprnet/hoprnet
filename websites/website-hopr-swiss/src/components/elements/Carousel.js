import React, { Children } from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'

const swipeThreshold = 40

const propTypes = {
  children: PropTypes.node,
  active: PropTypes.number,
  autorotate: PropTypes.bool,
  autorotateTiming: PropTypes.number,
}

const defaultProps = {
  children: null,
  active: null,
  autorotate: false,
  autorotateTiming: 7000,
}

class Carousel extends React.Component {
  state = {
    items: [],
    activeItem: this.props.active || 0,
    autorotateInterval: null,
    touchStartX: 0,
  }

  carousel = React.createRef()

  componentDidMount() {
    this.setState(
      {
        items: [...Array.prototype.slice.call(this.carousel.current.childNodes)],
      },
      () => this.heightFix()
    )
    this.playAutorotate()
    window.addEventListener('resize', this.heightFix)
  }

  componentWillUnmount() {
    window.removeEventListener('resize', this.heightFix)
    this.stopAutorotate()
  }

  goTo = (n, stop = false) => {
    stop && this.stopAutorotate()
    this.setState({ activeItem: n })
  }

  goToNext = (stop = false) => {
    let nextItem = this.state.activeItem + 1 >= this.props.children.length ? 0 : this.state.activeItem + 1
    this.goTo(nextItem, stop)
  }

  goToPrev = (stop = false) => {
    let prevItem = this.state.activeItem - 1 < 0 ? this.props.children.length - 1 : this.state.activeItem - 1
    this.goTo(prevItem, stop)
  }

  playAutorotate = () => {
    if (!this.state.autorotateInterval && this.props.autorotate) {
      this.setState({
        autorotateInterval: setInterval(() => {
          this.goToNext()
        }, this.props.autorotateTiming),
      })
    }
  }

  stopAutorotate = () => {
    clearInterval(this.state.autorotateInterval)
    this.setState({ autorotateInterval: null })
  }

  heightFix = () => {
    let taller = 0
    let height
    this.state.items.map(item => {
      item.classList.add('is-loading')
      height = item.offsetHeight
      item.classList.remove('is-loading')
      return (taller = height > taller ? height : taller)
    })
    this.carousel.current.style.minHeight = taller + 'px'
  }

  handleTouchStart = e => {
    this.setState({ touchStartX: e.changedTouches[0].screenX })
  }

  handleTouchEnd = e => {
    // If swipe is under the threshold, don't do anything.
    if (Math.abs(e.changedTouches[0].screenX - this.state.touchStartX) < swipeThreshold) return
    e.changedTouches[0].screenX < this.state.touchStartX ? this.goToNext(true) : this.goToPrev(true)
  }

  render() {
    const { className, children, active, autorotate, autorotateTiming, ...props } = this.props

    const classes = classNames('carousel-items', className)

    return (
      <React.Fragment>
        <div
          {...props}
          ref={this.carousel}
          className={classes}
          onTouchStart={this.handleTouchStart}
          onTouchEnd={this.handleTouchEnd}
        >
          {Children.map(children, (child, n) => {
            return React.cloneElement(child, {
              key: n,
              className: classNames(child.props.className, this.state.activeItem === n && 'is-active'),
            })
          })}
        </div>
        <div className="carousel-bullets">
          {Children.map(children, (child, n) => (
            <button
              key={n}
              className={classNames('carousel-bullet', this.state.activeItem === n && 'is-active')}
              onClick={this.goTo.bind(this, n, true)}
            ></button>
          ))}
        </div>
      </React.Fragment>
    )
  }
}

Carousel.propTypes = propTypes
Carousel.defaultProps = defaultProps

export default Carousel
