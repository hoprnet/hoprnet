import React, { useState, useEffect } from 'react'
import { useRouter } from 'next/router'
import Head from 'next/head'
import Menu from '../menu/menu'
import Modal from '../micro-components/modal'
import LeftSide from './left-side'
import RightSide from './right-side'
import CopieParagraph from '../data-view/copie-paragraph'
import DataBoxCloud from '../data-view/data-box-cloud'
import DataUpdateKnow from '../data-view/data-update-know'
import '../../styles/main.scss'
import api from '../../utils/api'

const Layout = ({ children }) => {
  const router = useRouter()
  const [darkMode, setDarkMode] = useState(false)
  const [modal, setModal] = useState(false)
  const [activaMenu, setactivaMenu] = useState(false)
  const [API_LastUpdated, SetAPI_LastUpdated] = useState(null)
  const [address, setAddress] = useState('')
  const [channel, setChannel] = useState('')
  const [hash, setHash] = useState('16Uiu2HAm7KxaBkgd9ENvhf5qAkp1c6Q5Q1dXe8HBDzxLN4SxAVw6')

  const copyCodeToClipboard = async () => {
    await navigator.clipboard.writeText(hash)
    setModal(true)
    setTimeout(() => {
      setModal(false)
    }, 4000)
  }

  const chenageMode = () => {
    if (!darkMode) {
      document.body.classList.add('dark-mode')
    } else {
      document.body.classList.remove('dark-mode')
    }
    setDarkMode(!darkMode)
  }

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData()
      if (response.data) {
        let CleanDate = response.data.refreshed.slice(0, -5)
        SetAPI_LastUpdated(CleanDate)
        setAddress(response.data.hoprCoverbotAddress)
        setChannel(response.data.hoprChannelContract)
        setHash(response.data.address)
      }
    }
    fetchData()
    // if(activaMenu){

    // }
  }, [])
  return (
    <>
      <Head>
        <title>hopr</title>
      </Head>
      <header>
        <nav className="navbar only-mobile-view">
          <div className="icon-logo">
            <img
              src={'/assets/brand/' + [darkMode ? 'logo_white.svg' : 'logo.svg']}
              className={[activaMenu ? 'open' : '']}
              alt="hopr"
            />
          </div>
          <div className={'icon-menu ' + [activaMenu ? 'open' : '']} onClick={() => setactivaMenu(!activaMenu)}>
            <span></span>
            <span></span>
            <span></span>
            <span></span>
            <span></span>
            <span></span>
          </div>
        </nav>
        {/*  */}
        <div className=" only-desktop-view ">
          <div className="icon-logo-desktop">
            <a href="https://hoprnet.org/" target="_blank" rel="noopener noreferrer">
              <img src={'/assets/brand/' + [darkMode ? 'logo_white.svg' : 'logo.svg']} alt="hopr" />
            </a>
          </div>
          <div className="active-darkmode">
            <label className="switch">
              <input type="checkbox" onChange={() => chenageMode()} />
              <span className="slider round">
                <img
                  className="icon-darkmode"
                  src={'/assets/icons/' + [darkMode ? 'luna.svg' : 'dom.svg']}
                  alt="hopr"
                />
              </span>
            </label>
          </div>
        </div>
      </header>
      <Menu darkMode={darkMode} chenageMode={chenageMode} activaMenu={activaMenu} hash={hash} copyCodeToClipboard={copyCodeToClipboard} />
      <div className="main-container">
        <div className="only-desktop-view">
          <LeftSide hash={hash} copyCodeToClipboard={copyCodeToClipboard} />
        </div>
        <section className={'about only-mobile-view ' + [router.pathname != '/' ? 'aux-margin' : '']}>
          <div className={[router.pathname != '/' ? 'only-desktop-view' : '']}>
            <CopieParagraph />
          </div>
        </section>
        {children}
        {/*  */}
        <section className="only-mobile-view">
          <hr />
          <DataBoxCloud address={address} channel={channel} />
          <hr />
          <DataUpdateKnow API_LastUpdated={API_LastUpdated} />
          <hr />
          <p className="paragraph-copy ">
            Thanks for helping us create the <span> HOPR network. </span>
          </p>
        </section>
        {/*  */}
        <RightSide address={address} channel={channel} API_LastUpdated={API_LastUpdated} />
        {/*  */}
      </div>
      <Modal modal={modal} hash={hash} />
    </>
  )
}

export default Layout
