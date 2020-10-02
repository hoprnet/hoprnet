import React from 'react'
import classNames from 'classnames'
import { ReactComponent as Twitter } from '../../assets/images/icons/twitter.svg'
import { ReactComponent as Telegram } from '../../assets/images/icons/telegram.svg'
import { ReactComponent as Linkedin } from '../../assets/images/icons/linkedin.svg'
import { ReactComponent as Github } from '../../assets/images/icons/github.svg'
import { ReactComponent as Medium } from '../../assets/images/icons/medium.svg'
import { ReactComponent as Youtube } from '../../assets/images/icons/youtube.svg'
import { ReactComponent as Discord } from '../../assets/images/icons/discord.svg'

const isCompany = false

const FooterSocial = ({ className, invertColor, ...props }) => {
  const classes = classNames('footer-social', className)
  // TODO: fix this
  const style = invertColor ? { fill: '#000050' } : { fill: 'white' }

  return (
    <div {...props} className={classes}>
      <ul className="list-reset">
        <li>
          <a href="https://twitter.com/hoprnet" target="_blank" rel="noopener noreferrer">
            <Twitter style={style} />
          </a>
        </li>
        <li>
          <a href="https://t.me/hoprnet" target="_blank" rel="noopener noreferrer">
            <Telegram style={style} />
          </a>
        </li>
        <li>
          <a
            href={`https://www.linkedin.com/company/${isCompany ? 'hoprswiss' : 'hoprnet'}`}
            target="_blank"
            rel="noopener noreferrer"
          >
            <Linkedin style={style} />
          </a>
        </li>
        <li>
          <a href="https://github.com/hoprnet" target="_blank" rel="noopener noreferrer">
            <Github style={style} />
          </a>
        </li>
        <li>
          <a href="https://medium.com/hoprnet" target="_blank" rel="noopener noreferrer">
            <Medium style={style} />
          </a>
        </li>
        <li>
          <a href="https://www.youtube.com/channel/UC2DzUtC90LXdW7TfT3igasA" target="_blank" rel="noopener noreferrer">
            <Youtube style={style} />
          </a>
        </li>
        <li>
          <a href="https://discord.gg/dEAWC4G" target="_blank" rel="noopener noreferrer">
            <Discord style={style} />
          </a>
        </li>
      </ul>
    </div>
  )
}

export default FooterSocial
