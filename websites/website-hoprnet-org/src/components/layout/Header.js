import React from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'
import { Link } from 'react-router-dom'
import Logo from './partials/Logo'

const propTypes = {
  active: PropTypes.bool,
  navPosition: PropTypes.string,
  hideNav: PropTypes.bool,
  hideSignin: PropTypes.bool,
  bottomOuterDivider: PropTypes.bool,
  bottomDivider: PropTypes.bool,
}

const defaultProps = {
  active: false,
  navPosition: '',
  hideNav: false,
  hideSignin: false,
  bottomOuterDivider: false,
  bottomDivider: false,
}

const isCompany = false

class Header extends React.Component {
  state = {
    isActive: false,
  }

  nav = React.createRef()
  hamburger = React.createRef()

  componentDidMount() {
    this.props.active && this.openMenu()
    document.addEventListener('keydown', this.keyPress)
    document.addEventListener('click', this.clickOutside)
  }

  componentWillUnmount() {
    document.removeEventListener('keydown', this.keyPress)
    document.addEventListener('click', this.clickOutside)
    this.closeMenu()
  }

  openMenu = () => {
    document.body.classList.add('off-nav-is-active')
    this.nav.current.style.maxHeight = this.nav.current.scrollHeight + 'px'
    this.setState({ isActive: true })
  }

  closeMenu = () => {
    document.body.classList.remove('off-nav-is-active')
    this.nav.current && (this.nav.current.style.maxHeight = null)
    this.setState({ isActive: false })
  }

  keyPress = e => {
    this.state.isActive && e.keyCode === 27 && this.closeMenu()
  }

  clickOutside = e => {
    if (!this.nav.current) return
    if (!this.state.isActive || this.nav.current.contains(e.target) || e.target === this.hamburger.current) return
    this.closeMenu()
  }

  render() {
    const {
      className,
      active,
      navPosition,
      hideNav,
      hideSignin,
      bottomOuterDivider,
      bottomDivider,
      hasBgColor,
      invertColor,
      sticky,
      ...props
    } = this.props

    const classes = classNames(
      'site-header cursor',
      bottomOuterDivider && 'has-bottom-divider',
      hasBgColor && 'has-bg-color',
      invertColor && 'invert-color',
      sticky && 'sticky',
      className
    )

    return (
      <header {...props} className={classes} style={{ backgroundColor: 'white' }}>
        <div className="container">
          <div className={classNames('site-header-inner', bottomDivider && 'has-bottom-divider')}>
            <Logo />
            {!hideNav && (
              <React.Fragment>
                <button
                  ref={this.hamburger}
                  className="header-nav-toggle"
                  onClick={this.state.isActive ? this.closeMenu : this.openMenu}
                >
                  <span className="screen-reader">Menu</span>
                  <span className="hamburger">
                    <span className="hamburger-inner"></span>
                  </span>
                </button>
                <nav ref={this.nav} className={classNames('header-nav', this.state.isActive && 'is-active')}>
                  <div className="header-nav-inner">
                    <ul className={classNames('list-reset text-ms', navPosition && `header-nav-${navPosition}`)}>
                      <li>
                        <Link to="/who-is-HOPR" onClick={this.closeMenu} style={{ textDecoration: 'none' }}>
                          ABOUT US
                        </Link>
                      </li>
                      {!isCompany && (
                        <>
                          <li>
                            <Link to="/layer0-data-privacy" onClick={this.closeMenu} style={{ textDecoration: 'none' }}>
                              TECHNOLOGY
                            </Link>
                          </li>
                          <li>
                            {/* <Link to="/node" onClick={this.closeMenu} style={{ textDecoration: 'none' }}>
                              JOIN TESTNET
                            </Link> */}
                            <a
                              href="http://saentis.hoprnet.org/"
                              target="_blank"
                              rel="noopener noreferrer"
                              style={{ textDecoration: 'none' }}
                            >
                              JOIN TESTNET
                            </a>
                          </li>
                          <li>
                            <Link
                              to="/do-business-with-HOPR"
                              onClick={this.closeMenu}
                              style={{ textDecoration: 'none' }}
                            >
                              BLOG
                            </Link>
                          </li>
                        </>
                      )}
                    </ul>
                    {!hideSignin && (
                      <ul className="list-reset header-nav-right">
                        <li>
                          <Link
                            to="/signup/"
                            className="button button-primary button-wide-mobile button-sm"
                            onClick={this.closeMenu}
                          >
                            Sign up
                          </Link>
                        </li>
                      </ul>
                    )}
                  </div>
                </nav>
              </React.Fragment>
            )}
          </div>
        </div>
      </header>
    )
  }
}

Header.propTypes = propTypes
Header.defaultProps = defaultProps

export default Header
