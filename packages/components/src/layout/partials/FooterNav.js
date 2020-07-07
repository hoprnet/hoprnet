import React from 'react'
import classNames from 'classnames'
import { Link } from 'react-router-dom'

const FooterNav = ({ className, ...props }) => {
  const classes = classNames('footer-nav', className)

  return (
    <nav {...props} className={classes}>
      <ul className="list-reset">
        <li>
          <Link to="/who-is-HOPR#contact">Contact</Link>
        </li>
        <li>
          <Link to="/who-is-HOPR#about">About us</Link>
        </li>
        {/* <li>
          <Link to="/faqs/" >
            FAQ's
          </Link>
        </li> */}
        <li>
          <Link to="/do-business-with-HOPR#for_you">Support</Link>
        </li>
        <li>
          <Link to="/disclaimer">Disclaimer</Link>
        </li>
      </ul>
    </nav>
  )
}

export default FooterNav
