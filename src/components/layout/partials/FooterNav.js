import React from 'react'
import classNames from 'classnames'
import { Link } from 'react-router-dom'

const FooterNav = ({ className, ...props }) => {
  const classes = classNames('footer-nav', className)

  return (
    <nav {...props} className={classes}>
      <ul className="list-reset">
        <li>
          <Link to="/HOPR/#contact" target="_blank" rel="noopener noreferrer">
            Contact
          </Link>
        </li>
        <li>
          <Link to="/HOPR/#about" target="_blank" rel="noopener noreferrer">
            About us
          </Link>
        </li>
        {/* <li>
          <Link to="/faqs/" target="_blank" rel="noopener noreferrer">
            FAQ's
          </Link>
        </li> */}
        <li>
          <Link to="/for_you/#for_you" target="_blank" rel="noopener noreferrer">
            Support
          </Link>
        </li>
      </ul>
    </nav>
  )
}

export default FooterNav
