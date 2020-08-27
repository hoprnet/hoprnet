import Hopr from "@hoprnet/hopr-core";
import type { HoprOptions } from "@hoprnet/hopr-core";
import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import { commands } from '@hoprnet/hopr-chat'
import {LogStream, Socket} from './logs'
import express from 'express'
import http from 'http'
import fs from 'fs'
import ws from 'ws'
import path from 'path'
import debug from 'debug'

let debugLog = debug('hoprd:admin')

export function setupAdminServer(logs: LogStream, node: Hopr<HoprCoreConnector>){
  let cmds = new commands.Commands(node)
  var app = express()
  app.get('/', function(req, res){
    res.set('Content-Type', 'text/html')
    res.send(fs.readFileSync(path.resolve('./src/admin.html')))
  })

  const server = http.createServer(app);

  const wsServer = new ws.Server({ server: server });
  wsServer.on('connection', socket => {
    socket.on('message', message => {
      debugLog("Message from client", message)
      logs.logFullLine(`admin > ${message}`)
      cmds.execute(message.toString()).then( (resp) => {
        if (resp) {
          logs.logFullLine(resp)
        }
      })
      // TODO
    });
    socket.on('error', err => {
      debugLog('Error', err)
      logs.log('Websocket error', err.toString())
    })
    logs.subscribe(socket)
  });

  const port = process.env.HOPR_ADMIN_PORT || 3000
  server.listen(port)
  logs.log('Admin server listening on port '+ port)
}
