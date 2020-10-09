// Hacks based on
// https://github.com/MetaMask/metamask-extension/blob/develop/ui/app/components/ui/jazzicon/jazzicon.component.js
import React, { createRef, PureComponent } from 'react'
import PropTypes from 'prop-types'
import dynamic from 'next/dynamic'
import jazzicon from 'jazzicon'
import md5 from 'tiny-hashes/md5'

/**
 * Wrapper around the jazzicon library to return a React component, as the library returns an
 * HTMLDivElement which needs to be appended.
 */
export default class Jazzicon extends PureComponent {
  static propTypes = {
    address: PropTypes.string.isRequired,
    className: PropTypes.string,
    diameter: PropTypes.number,
    style: PropTypes.object
  }

  static defaultProps = {
    diameter: 46
  }

  container = createRef()

  componentDidMount() {
    this.appendJazzicon()
  }

  componentDidUpdate(prevProps) {
    const { address: prevAddress, diameter: prevDiameter } = prevProps
    const { address, diameter } = this.props

    if (address !== prevAddress || diameter !== prevDiameter) {
      this.removeExistingChildren()
      this.appendJazzicon()
    }
  }

  removeExistingChildren() {
    const { children } = this.container.current

    for (let i = 0; i < children.length; i++) {
      this.container.current.removeChild(children[i])
    }
  }

  appendJazzicon() {
    if (typeof window !== 'undefined') {
      const { address, diameter } = this.props
      // NB: 'goodenough' transform between B58 string and js int
      const image = jazzicon(diameter, parseInt(md5(address).slice(-10), 16))
      this.container.current.appendChild(image)
    }
  }

  render() {
    const { className, style } = this.props

    return <div className={className} ref={this.container} style={style} />
  }
}
