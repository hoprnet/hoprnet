import React from 'react'
import classNames from 'classnames'
import { ReactComponent as Twitter } from '../../assets/images/icons/twitter.svg'
import { ReactComponent as Telegram } from '../../assets/images/icons/telegram.svg'
import { ReactComponent as Linkedin } from '../../assets/images/icons/linkedin.svg'
import { ReactComponent as Github } from '../../assets/images/icons/github.svg'

const isCompany = process.env.REACT_APP_IS_COMPANY === 'TRUE'

const FooterSocial = ({ className, ...props }) => {
  const classes = classNames('footer-social', className)

  return (
    <div {...props} className={classes}>
      <ul className="list-reset">
        <li>
          <a href="https://twitter.com/hoprnet" target="_blank" rel="noopener noreferrer">
            <Twitter />
          </a>
        </li>
        <li>
          <a href="https://t.me/hoprnet" target="_blank" rel="noopener noreferrer">
            <Telegram />
          </a>
        </li>
        <li>
          <a
            href={`https://www.linkedin.com/company/${isCompany ? 'hoprswiss' : 'hoprnet'}`}
            target="_blank"
            rel="noopener noreferrer"
          >
            <Linkedin />
          </a>
        </li>
        <li>
          <a href="https://github.com/hoprnet" target="_blank" rel="noopener noreferrer">
            <Github />
          </a>
        </li>
      </ul>
    </div>
  )
}

export default FooterSocial
