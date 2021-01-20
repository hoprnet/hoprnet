/*
 * Maintain a websocket connection
 */

const MAX_MESSAGES_CACHED = 50

export class Connection {
  logs = []
  prevLog = ''

  constructor(setConnecting, setMessages, setConnectedPeers) {
    this.setConnecting = setConnecting
    this.setMessages = setMessages
    this.setConnectedPeers = setConnectedPeers
    this.connect()
  }

  appendMessage(event) {
    try {
      const msg = JSON.parse(event.data)
      if (msg.type == 'log') {
        if (this.logs.length > MAX_MESSAGES_CACHED) {
          // Avoid memory leak
          this.logs.splice(0, this.logs.length - MAX_MESSAGES_CACHED) // delete elements from start
        }
        this.logs.push(msg)
        this.setMessages(this.logs.slice(0)) // Need a clone
      } else if (msg.type == 'connected') {
        this.setConnectedPeers(msg.msg.split(','))
      } else if (msg.type == 'fatal-error') {
        this.setConnecting('true')
        this.logs.push(msg)

        // Let's elaborate on certain error messages:
        if (msg.msg.indexOf('account has no funds') > -1) {
          this.logs.push({ msg: '- Please send 0.1 ETH to the account', ts: new Date().toISOString() })
          this.logs.push({ msg: '- Then restart the node', ts: new Date().toISOString() })
        }

        this.setMessages(this.logs.slice(0)) // Need a clone
      }
    } catch (e) {
      console.log('ERR', e)
    }
  }

  connect() {
    console.log('Connecting ...')
    var client = new WebSocket('ws://' + window.location.host)
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
      console.log(event)
    }

    client.onerror = (error) => {
      console.log('Connection error:', error)
    }

    client.onclose = () => {
      console.log('Web socket closed')
      this.setConnecting(true)
      this.appendMessage(' --- < Lost Connection, attempting to reconnect... > ---')
      var self = this
      setTimeout(function () {
        try {
          self.connect()
          console.log('connection')
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
