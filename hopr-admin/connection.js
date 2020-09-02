/*
 * Maintain a websocket connection
 */

export class Connection {
  messages = []
  prevLog = ""

  constructor(
    setConnecting,
    setMessages
  ){
    this.setConnecting = setConnecting
    this.setMessages = setMessages
    this.connect()
  }

  appendMessage(event) {
    this.messages.push(event.data)
    this.setMessages(this.messages.slice(0)) // Need a clone
  }

  connect() {
    console.log('Connecting ...')
    var client = new WebSocket('ws://' + window.location.host);
    console.log('Web socket created')

    client.onopen = () => {
      console.log('Web socket opened')
      this.setConnecting(false)

      document.querySelector('#command').onkeydown = (e) => {
        if (e.keyCode == 13 ) { // enter 
          var text = e.target.value 
          console.log("Command: ", text)
          if (text.length > 0) {
            client.send(text)
            this.prevLog = text
            e.target.value = ""
          }
        }
        if (e.keyCode == 38) { // Up Arrow
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
      setTimeout(function(){
        try {
          connect()
          console.log('connection')
        } catch (e){
          console.log('Error connecting', e)
        }
      }, 1000);
    }
  }

  disconnect(){
    if (this.client) {
      this.client.close()
    }
  }
}
