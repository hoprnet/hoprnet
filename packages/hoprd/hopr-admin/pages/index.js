import React, { useEffect, useState, KeyboardEvent } from 'react'
import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Logo from '../components/logo'
import { Logs } from '../components/log'
import { Connection } from '../connection'
import dynamic from 'next/dynamic'
import Cookies from 'js-cookie'
import { accountWithdraw, getAddresses, getBalances } from '../fetch/account'
import { parseCmd } from '../fetch/client'
import { getAliases, setAliases } from '../fetch/aliases'
import { closeChannel, getChannels, getTickets, redeemTickets, setChannels } from '../fetch/channels'
import { getNodeInfo, getNodeVer, pingNodePeer } from '../fetch/node'
import { getSettings } from '../fetch/settings'
import { sendMessage, signAddress } from '../fetch/messages'

const Jazzicon = dynamic(() => import('../components/jazzicon'), { ssr: false })
const gitHash = process.env.NEXT_PUBLIC_GIT_COMMIT

class TokenInput extends React.Component {
  constructor(props) {
    super(props)
    this.handleKeyPress = this.handleKeyPress.bind(this)
  }

  handleKeyPress(e) {
    if (e.key == 'Enter') {
      var text = e.target.value
      Cookies.set('X-Auth-Token', text)
      this.props.handleTokenSet()
    }
  }

  render() {
    const tokenCookie = Cookies.get('X-Auth-Token')
    return tokenCookie === undefined ? (
      <div className="send">
        <input
          className="token"
          onKeyPress={this.handleKeyPress}
          id="token"
          type="password"
          placeholder="security token"
        />
      </div>
    ) : null
  }
}

export default function Home() {
  let connection

  const [showConnected, setShowConnected] = useState(false)
  const [connecting, setConnecting] = useState(true)
  const [ready, setReady] = useState(false)
  const [messages, setMessages] = useState([]) // The fetish for immutability in react means this will be slower than a mutable array..
  const [peers, setConnectedPeers] = useState([])
  const [, updateState] = React.useState()
  const handleAuthFailed = React.useCallback(() => {
    Cookies.remove('X-Auth-Token')
    setAuthFailed(true)
  }, [])
  const [authFailed, setAuthFailed] = useState(false)
  const handleTokenSet = React.useCallback(() => {
    connection.connect()
    setAuthFailed(false)
    updateState({})
  }, [])

  useEffect(() => {
    // if (typeof window !== 'undefined') {
    //   connection = new Connection(setConnecting, setReady, setMessages, setConnectedPeers, handleAuthFailed)
    //   return Connection.disconnect
    // }

    if (typeof window !== 'undefined') {
      document.querySelector('#command').onkeydown = (e) => {
        // enter
        if (e.keyCode == 13) {
          var text = e.target.value
          if (text.length > 0) {
            const userInput = parseCmd(text)
            let options = []
            if (userInput.query != '') {
              options = userInput.query.trim().split(/\s+/)
            }
            switch (userInput.cmd){
              // Test cmd: withdraw 1337 NATIVE 0xEA9eDAE5CfC794B75C45c8fa89b605508A03742a
              case "withdraw":
                accountWithdraw({
                  "amount": options[0],
                  "currency": options[1],
                  "recipient": options[2]
                })
                break
              case "balance":
                getBalances()
                break
              case "address":
                getAddresses()
                break
              case "alias":
                // FIXME: setAliases not working (debug)
                // Test cmd: alias 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 Alice
                if (options.length) {
                  setAliases(options[0], options[1]);
                }else {
                  getAliases();
                }
                break
              case "channels":
                getChannels()
                break
              case "close":
                // Test cmd: close 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12
                closeChannel(options[0])
                break
              case "info":
                getNodeInfo()
                break
              case "open":
                // Test cmd: open 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 1000000
                // FIXME: debug UNKNOWN_FAILURE
                if (options.length){
                  setChannels(options[0], options[1]);
                }
                break
              case "redeemTickets":
                // Test cmd: redeemTickets 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12
                // FIXME: Debug err code 422
                redeemTickets(options[0])
                break
              case "tickets":
                // Test cmd: tickets 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12
                getTickets(options[0])
                break
              case "version":
                getNodeVer()
                break
              case "ping":
                // Test cmd: ping 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12
                pingNodePeer(options[0])
                break
              case "settings":
                getSettings();
                break
              case "sign":
                // Test cmd: sign 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12
                signAddress(options[0]);
                break
              case "send":
                // Test cmd: send Hello 16Uiu2HAm2SF8EdwwUaaSoYTiZSddnG4hLVF7dizh32QFTNWMic2b [16Uiu2HAm1uV82HyD1iJ5DmwJr4LftmJUeMfj8zFypBRACmrJc16n]
                // FIXME: 400 Bad request
                // console.log(options)
                sendMessage(options[0], options[1], options[2])
              case "peers":
                // TODO: Use /node/info??
                break
              case "quit":
                // TODO: Find out how
                break
              default:
                console.log("Command Not found")
            }
            e.target.value = ''
          }
        }
      }
    }


  }, [])

  const cookie = Cookies.get('X-Auth-Token')

  return (
    <div className={styles.container}>
      <Head>
        <title>HOPR Admin</title>
      </Head>

      <Logo onClick={() => setShowConnected(!showConnected)} />
      <h1>HOPR Logs - {gitHash ? gitHash : '*'}</h1>

      <Logs messages={messages} connecting={connecting} authRequired={authFailed} />

      <div className="send">
        <input id="command" type="text" autoFocus placeholder="type 'help' for full list of commands" />
      </div>

      {(authFailed || cookie === null) && <TokenInput handleTokenSet={handleTokenSet} />}

      {showConnected && (
        <div className={styles.connectedPeers}>
          <h2>Connected Peers ({peers.length})</h2>
          <div className={styles.connectedPeersList}>
            {peers.map((x) => (
              <div className={styles.peer} key={x}>
                <Jazzicon diameter={40} address={x} className={styles.peerIcon} />
                <div>{x}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
