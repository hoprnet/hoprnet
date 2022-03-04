/*
 * Maintain a websocket connection
 */

import Cookies from 'js-cookie'
import { parseCmd } from './client'
import {
  accountWithdraw,
  closeChannel,
  getAddresses,
  getAliases,
  getBalances,
  getChannels,
  getNodeInfo, getNodeVer, getSettings, getTickets, pingNodePeer, redeemTickets, sendMessage,
  setAliases, setChannels, signAddress
} from './fetch'
// import ListCommands from '../lib/commands/listCommands'

const MAX_MESSAGES_CACHED = 50

export class Connection {
  logs = []
  prevLog = ''
  authFailed = false

  constructor(setConnecting, setReady, setMessages, setConnectedPeers, onAuthFailed) {
    this.setConnecting = setConnecting
    this.setReady = setReady
    this.setMessages = setMessages
    this.setConnectedPeers = setConnectedPeers
    this.onAuthFailed = onAuthFailed
    this.connect()
  }

  appendMessage(event) {
    if (event.data === undefined) {
      return
    }

    try {
      const msg = JSON.parse(event.data)

      this.authFailed = false

      switch (msg.type) {
        case 'log':
          if (this.logs.length > MAX_MESSAGES_CACHED) {
            // Avoid memory leak
            this.logs.splice(0, this.logs.length - MAX_MESSAGES_CACHED) // delete elements from start
          }
          this.logs.push(msg)
          this.setMessages(this.logs.slice(0)) // Need a clone
          break
        case 'connected':
          this.setConnectedPeers(msg.msg.split(','))
          break
        case 'fatal-error':
          this.logs.push(msg)

          // Let's elaborate on certain error messages:
          if (msg.msg.indexOf('account has no funds') > -1) {
            this.logs.push({ msg: '- Please send 0.1 gETH to the account', ts: new Date().toISOString() })
            this.logs.push({ msg: '- Then restart the node', ts: new Date().toISOString() })
          }

          this.setMessages(this.logs.slice(0)) // Need a clone
          break
        case 'status':
          if (msg.msg === 'READY') {
            this.setReady(true)
          } else {
            this.setReady(false)
          }
          break
        case 'auth-failed':
          this.logs.push(msg)
          this.authFailed = true
          this.setConnecting(false)
          this.onAuthFailed()
          break
      }
    } catch (e) {
      console.log('ERR', e)
    }
  }

  async connect() {
    console.log('Connecting ...')
    var client
    try {
      // See https://stackoverflow.com/a/55487820
      var client = navigator.clipboard
        ? await fetch(`https://${window.location.host}/api/ssl`).then(
            (_) => new WebSocket('wss://' + window.location.host)
          )
        : new WebSocket('ws://' + 'localhost:19501/')
    } catch (err) {
      console.log('Invalid SSL or non-SSL support')
      client = new WebSocket('ws://' + "localhost:19501/")
    }
    console.log('Web socket created')

    client.onopen = () => {
      console.log('Web socket opened')
      this.setConnecting(false)

      document.querySelector('#command').onkeydown = (e) => {
        // enter
        if (e.keyCode == 13) {
          var text = e.target.value
          if (text.length > 0) {
            // client.send(text)
            // this.prevLog = text

            const userInput = parseCmd(text)
            let options = []
            if (userInput.query != '') {
              options = userInput.query.trim().split(/\s+/)
            }
            switch (userInput.cmd) {
              // Test cmd: withdraw 1337 NATIVE 0xEA9eDAE5CfC794B75C45c8fa89b605508A03742a
              case "withdraw":
                accountWithdraw({
                  "amount": options[0],
                  "currency": options[1],
                  "recipient": options[2]
                })
                break
              case "balance":
                getBalances().then(balances => {
                  this.logs.push({type: "log", msg: `${balances.native}`, ts: ""})
                  this.setMessages(this.logs.slice(0)) // Need a clone
                })
                break
              case "address":
                getAddresses()
                break
              case "alias":
                // FIXME: setAliases not working (debug)
                // Test cmd: alias 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12 Alice
                if (options.length) {
                  setAliases(options[0], options[1]);
                } else {
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
                if (options.length) {
                  setChannels(options[0], options[1]);
                }
                break
              case "redeemTickets":
                // Test cmd: redeemTickets 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12
                // FIXME: Debug err code 422
                redeemTickets()
                break
              case "tickets":
                // Test cmd: tickets 16Uiu2HAmUsJwbECMroQUC29LQZZWsYpYZx1oaM1H9DBoZHLkYn12
                getTickets()
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
                // TODO: See https://github.com/hoprnet/hoprnet/pull/3617
                break
              case "help":
                client.send("help")
                // const listcmd = ListCommands()
                // listcmd.execute()
                break
              default:
                console.log("Command not found.")
                break
            }
            e.target.value = ''
          }
        }
        if (e.keyCode == 38) {
          // Up Arrow
          e.target.value = this.prevLog
        }
      }
    }

    client.onmessage = (event) => {
      this.appendMessage(event)
    }

    client.onerror = (error) => {
      console.log('Connection error:', error)
      this.setConnecting(false)
    }

    client.onclose = () => {
      console.log('Web socket closed')
      this.appendMessage(' --- < Lost Connection, attempting to reconnect... > ---')
      var self = this

      setTimeout(function () {
        try {
          if (!self.authFailed) {
            this.setConnecting(true)
            self.connect()
          }
        } catch (e) {
          console.log('Error connecting', e)
        }
      }, 1000)
    }
  }

  disconnect() {
    if (this.client) {
      this.client.close()
    }
  }
}
