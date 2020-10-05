import React from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'

const propTypes = {
  children: PropTypes.node,
  active: PropTypes.bool,
  title: PropTypes.string.isRequired,
}

const defaultProps = {
  children: null,
  active: false,
  title: '',
}

class AccordionItem extends React.Component {
  state = {
    isActive: false,
  }

  content = React.createRef()

  componentDidMount() {
    this.props.active && this.openItem()
  }

  openItem = () => {
    this.content.current.style.maxHeight = this.content.current.scrollHeight + 'px'
    this.setState({ isActive: true })
  }

  closeItem = () => {
    this.content.current.style.maxHeight = null
    this.setState({ isActive: false })
  }

  render() {
    const { className, children, active, title, ...props } = this.props

    const classes = classNames(this.state.isActive && 'is-active', className)

    return (
      <li {...props} className={classes}>
        <div className="accordion-header text-sm" onClick={this.state.isActive ? this.closeItem : this.openItem}>
          <span className="h6 m-0">{title}</span>
          <div className="accordion-icon"></div>
        </div>
        <div ref={this.content} className="accordion-content text-xs">
          <p>{children}</p>
        </div>
      </li>
    )
  }
}

AccordionItem.propTypes = propTypes
AccordionItem.defaultProps = defaultProps

export default AccordionItem
