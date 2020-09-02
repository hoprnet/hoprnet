import ws from 'ws'
import debug from 'debug'

export type Socket = ws

let debugLog = debug('hoprd')

const MAX_MESSAGES_CACHED = 100

type Message = {
  msg: string,
  ts: string
}

export class LogStream {
  private messages: Message[] = []
  private connections: Socket[] = []

  constructor(){
  }

  subscribe(sock: Socket){
    this.connections.push(sock);
    this.messages.forEach(m => this._sendMessage(m, sock))
  }

  log(...args: string[]){
    const msg = `${args.join(' ')}`
    this._log(msg)
  }

  logFullLine(...args: string[]){
    const msg = `${args.join(' ')}`
    this._log(msg)
  }

  _log(msg: string){
    debugLog(msg) 
    let m = {msg: msg, ts: new Date().toISOString()}
    this.messages.push(m)
    if (this.messages.length > MAX_MESSAGES_CACHED){ // Avoid memory leak
      this.messages.splice(0, this.messages.length - MAX_MESSAGES_CACHED); // delete elements from start
    }
    this.connections.forEach((conn: Socket, i: number) => {
      if (conn.readyState == ws.OPEN) {
        this._sendMessage(m, conn)
      } else {
        // Handle bad connections:
        if (conn.readyState !== ws.CONNECTING) {
          // Only other possible states are closing or closed
          this.connections.splice(i, 1)
        }

      }
    })
  }

  _sendMessage(m: Message, s: Socket) {
    s.send(JSON.stringify(m))
  }
}

