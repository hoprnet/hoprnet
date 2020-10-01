import React from 'react'
import PropTypes from 'prop-types'
import classNames from 'classnames'
import Logo from './partials/LogoInvert'
import FooterNav from './partials/FooterNav'
import FooterSocial from './partials/FooterSocial'

const propTypes = {
  topOuterDivider: PropTypes.bool,
  topDivider: PropTypes.bool,
}

const defaultProps = {
  topOuterDivider: false,
  topDivider: false,
}

const isCompany = true

class Footer extends React.Component {
  render() {
    const { className, topOuterDivider, topDivider, ...props } = this.props

    const classes = classNames(
      'site-footer invert-color center-content-mobile',
      topOuterDivider && 'has-top-divider',
      className
    )

    return (
      <footer {...props} className={classes}>
        <div className="container">
          <div className={classNames('site-footer-inner', topDivider && 'has-top-divider')}>
            <div className="footer-top space-between text-xxs">
              <Logo />
              <FooterSocial />
            </div>
            <div className="footer-bottom space-between text-xxs invert-order-desktop">
              <FooterNav />
              <div className="footer-copyright">
                &copy; 2020 HOPR {isCompany ? 'Services AG' : 'Association'}, all rights reserved
              </div>
            </div>
          </div>
        </div>
      </footer>
    )
  }
}

Footer.propTypes = propTypes
Footer.defaultProps = defaultProps

export default Footer
