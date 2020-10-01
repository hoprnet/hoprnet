import React from 'react'
import classNames from 'classnames'
import { SectionProps } from '../utils/SectionProps'
import Image from '../elements/Image'
import Modal from '../elements/Modal'

const propTypes = {
  ...SectionProps.types,
}

const defaultProps = {
  ...SectionProps.defaults,
}

class HeroFull extends React.Component {
  state = {
    videoModalActive: false,
  }
  openModal = e => {
    e.preventDefault()
    this.setState({ videoModalActive: true })
  }

  closeModal = e => {
    e.preventDefault()
    this.setState({ videoModalActive: false })
  }

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
      'hero section center-content',
      topOuterDivider && 'has-top-divider',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      className
    )

    const innerClasses = classNames(
      'hero-inner section-inner',
      topDivider && 'has-top-divider',
      bottomDivider && 'has-bottom-divider'
    )

    return (
      <section {...props} className={outerClasses}>
        <div className="container-sm">
          <div className={innerClasses}>
            <div className="hero-content">
              <div className="container-sm">
                <h1 className="m-0 reveal-from-top">Engage Your Visitors with a beautiful template</h1>
              </div>
            </div>
            <div className="hero-figure reveal-from-bottom" data-reveal-delay="150">
              <a
                data-video="https://player.vimeo.com/video/174002812"
                href="#0"
                aria-controls="video-modal"
                onClick={this.openModal}
              >
                <Image
                  className="has-shadow"
                  src={require('./../assets/images/video-placeholder.svg')}
                  alt="Video"
                  width={712}
                  height={400}
                />
              </a>
            </div>
            <Modal
              id="video-modal"
              show={this.state.videoModalActive}
              handleClose={this.closeModal}
              video="https://player.vimeo.com/video/174002812"
              videoTag="iframe"
            />
          </div>
        </div>
      </section>
    )
  }
}

HeroFull.propTypes = propTypes
HeroFull.defaultProps = defaultProps

export default HeroFull
