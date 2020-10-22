import React, { useState, useEffect } from 'react'
import { useRouter } from 'next/router'
import Link from 'next/link'
import '../../styles/main.scss'
import TweetBasodino from '../tweet-basodino'
import api from '../../utils/api'

const Menu = ({ activaMenu }) => {
  const router = useRouter()
  const [hash, setHash] = useState('16Uiu2HAm7KxaBkgd9ENvhf5qAkp1c6Q5Q1dXe8HBDzxLN4SxAVw6')

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData()
      if (response.data) setHash(response.data.address)
    }
    fetchData()
  }, [])

  const [modal, setModal] = useState(false)
  const copyCodeToClipboard = () => {
    navigator.clipboard.writeText(hash)
    setModal(true)
    setTimeout(() => {
      setModal(false)
    }, 4000)
  }

  return (
    <>
      <div className={'menu-mobile ' + [activaMenu ? 'open-menu' : '']}>
        <div className="menu-container">
          <div>
            <ul>
              <Link href="/">
                <li className={[router.pathname == '/' ? 'active' : '']}>
                  <img src="/assets/icons/home.svg" alt="hopr HOME" />
                  <p>HOME</p>
                </li>
              </Link>
              <Link href="/hopr-allocation">
                <li className={[router.pathname == '/hopr-allocation' ? 'active' : '']}>
                  <img src="/assets/icons/horp_icon.svg" alt="hopr HOPR ALLOCATION" />
                  <p>HOPR ALLOCATION</p>
                </li>
              </Link>
              <Link href="https://discord.com/invite/wUSYqpD" target="_blank">
                <li>
                  <img src="/assets/icons/discord.svg" alt="hopr DISCORD" />
                  <p>DISCORD</p>
                </li>
              </Link>
              <Link href="/help">
                <li className={[router.pathname == '/help' ? 'active' : '']}>
                  <img src="/assets/icons/help.svg" alt="hopr HELP" />
                  <p>HELP</p>
                </li>
              </Link>
            </ul>

            <hr />
            <div className="quick-code">
              <p>HOPR node</p>
              <div className="hash" onClick={() => copyCodeToClipboard()}>
                <p>{hash}</p>
                <div>
                  <img src="/assets/icons/copy.svg" alt="copy" />
                </div>
              </div>
            </div>
            <hr />
            <div className="twitter-line-menu">
              <div>
                <a href="https://twitter.com/hoprnet" target="_blank">
                  <img src="/assets/icons/twitter.svg" alt="twitter" />
                  <p>@hoprnet</p>
                </a>
              </div>
              <div>
                <TweetBasodino>
                  <img src="/assets/icons/twitter.svg" alt="twitter" /> <p>#Basodino</p>
                </TweetBasodino>
              </div>
            </div>
          </div>
        </div>
        {/*  */}
      </div>
      <div className={'modal-copy-menu ' + [modal ? 'show-modal-menu' : '']}>
        <div className="box-modal-copy">
          <div className="icon-logo">
            <img src="/assets/brand/logo.svg" alt="hopr" />
          </div>
          <div className="content">
            <div>
              <p>{hash}</p>
            </div>
            <h5>copied to clipboard</h5>
            <hr className="hr-alert" />
            <p className="copy-alert">
              this message is only informative it <br />
              closes itself in <span>4 seconds.</span>
            </p>
          </div>
        </div>
      </div>
    </>
  )
}

export default Menu
