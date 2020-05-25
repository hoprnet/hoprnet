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

class Footer extends React.Component {
  render() {
    const { className, topOuterDivider, topDivider, ...props } = this.props

    const classes = classNames('site-footer invert-color', topOuterDivider && 'has-top-divider', className)

    return (
      <footer {...props} className={classes}>
        <div className="container">
          <div className={classNames('site-footer-inner', topDivider && 'has-top-divider')}>
            <div className="footer-top text-xxs">
              <div className="footer-blocks">
                <div className="footer-block">
                  <Logo className="mb-16" />
                  <div className="footer-copyright">&copy; 2020 HOPR Association, all rights reserved</div>
                </div>
                <div className="footer-block">
                  <div className="footer-block-title">Company</div>
                  <ul className="list-reset mb-0">
                    <li>
                      <a href="https://cruip.com/">Dummy text used</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">The purpose of lorem</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">Filler text can be very useful</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">Be on design</a>
                    </li>
                  </ul>
                </div>
                <div className="footer-block">
                  <div className="footer-block-title">Uses cases</div>
                  <ul className="list-reset mb-0">
                    <li>
                      <a href="https://cruip.com/">Consectetur adipiscing</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">Lorem Ipsum is place</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">Excepteur sint</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">Occaecat cupidatat</a>
                    </li>
                  </ul>
                </div>
                <div className="footer-block">
                  <div className="footer-block-title">Docs</div>
                  <ul className="list-reset mb-0">
                    <li>
                      <a href="https://cruip.com/">The purpose of lorem</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">Dummy text used</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">Excepteur sint</a>
                    </li>
                    <li>
                      <a href="https://cruip.com/">Occaecat cupidatat</a>
                    </li>
                  </ul>
                </div>
              </div>
            </div>
            <div className="footer-bottom space-between center-content-mobile text-xxs">
              <FooterNav />
              <FooterSocial />
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
