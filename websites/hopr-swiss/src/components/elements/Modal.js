import React from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'

const propTypes = {
  children: PropTypes.node,
  handleClose: PropTypes.func.isRequired,
  show: PropTypes.bool.isRequired,
  closeHidden: PropTypes.bool,
  video: PropTypes.string,
  videoTag: PropTypes.oneOf(['iframe', 'video']),
}

const defaultProps = {
  children: null,
  show: false,
  closeHidden: false,
  video: '',
  videoTag: 'iframe',
}

class Modal extends React.Component {
  state = {}

  componentDidMount() {
    document.addEventListener('keydown', this.keyPress)
    document.addEventListener('click', this.stopProgagation)
  }

  componentWillUnmount() {
    document.removeEventListener('keydown', this.keyPress)
    document.removeEventListener('click', this.stopProgagation)
  }

  componentDidUpdate(prevProps) {
    prevProps.show !== this.props.show && this.handleBodyClass()
  }

  handleBodyClass = () => {
    if (document.querySelectorAll('.modal.is-active').length) {
      document.body.classList.add('modal-is-active')
    } else {
      document.body.classList.remove('modal-is-active')
    }
  }

  keyPress = e => {
    e.keyCode === 27 && this.props.handleClose(e)
  }

  stopProgagation = e => {
    e.stopPropagation()
  }

  render() {
    const { className, children, handleClose, show, closeHidden, video, videoTag, ...props } = this.props

    const classes = classNames('modal', show && 'is-active', video && 'modal-video', className)

    return (
      <React.Fragment>
        {show && (
          <div {...props} className={classes} onClick={handleClose}>
            <div className="modal-inner" onClick={this.stopProgagation}>
              {video ? (
                <div className="responsive-video">
                  {videoTag === 'iframe' ? (
                    <iframe title="video" src={video} frameBorder="0" allowFullScreen></iframe>
                  ) : (
                    <video v-else controls src={video}></video>
                  )}
                </div>
              ) : (
                <React.Fragment>
                  {!closeHidden && <button className="modal-close" aria-label="close" onClick={handleClose}></button>}
                  <div className="modal-content">{children}</div>
                </React.Fragment>
              )}
            </div>
          </div>
        )}
      </React.Fragment>
    )
  }
}

Modal.propTypes = propTypes
Modal.defaultProps = defaultProps

export default Modal
