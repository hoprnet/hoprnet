/*
 * Maintain a websocket connection
 */

import Cookies from 'js-cookie'

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
        : new WebSocket('ws://' + window.location.host)
    } catch (err) {
      console.log('Invalid SSL or non-SSL support')
      client = new WebSocket('ws://' + window.location.host)
    }
    console.log('Web socket created')

    client.onopen = () => {
      console.log('Web socket opened')
      this.setConnecting(false)

      document.querySelector('#command').onkeydown = (e) => {
        if (e.keyCode == 13) {
          // enter
          var text = e.target.value
          console.log('Command: ', text)
          if (text.length > 0) {
            client.send(text)
            this.prevLog = text
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
