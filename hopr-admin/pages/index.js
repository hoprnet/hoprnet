import React, { useEffect, useState } from "react";
import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Logo from '../components/logo'

function browser(){
  console.log("Mounting")
  var prevLog = ""

  var appendLog = function(msg){
    var logs = document.querySelector('#log')
    logs.textContent = logs.textContent + '\n'+ msg
    logs.scrollIntoView({block: 'end', behaviour: 'smooth'});
  }

  var connect = function(){
    var logs = document.querySelector('.logs')
    console.log('Connecting ...')
    var client = new WebSocket('ws://' + window.location.host);
    console.log('Web socket created')

    client.onopen = function(){
      console.log('Web socket opened')
      logs.classList.remove('connecting')
      document.querySelector('#command').disabled = false
      document.querySelector('#command').focus()

      document.querySelector('#command').onkeydown = function(e) {
        if (e.keyCode == 13 ) { // enter 
          var text = e.target.value 
          console.log("Command: ", text)
          if (text.length > 0) {
            client.send(text)
            prevLog = text
            e.target.value = ""
          }
        }
        if (e.keyCode == 38) { // Up Arrow
          e.target.value = prevLog
        }
      }

    }

    client.onmessage = function(event) {
      appendLog(event.data)
      console.log(event)
    }

    client.onerror = function(error){
      console.log('Connection error:', error)
    }

    client.onclose = function(){
      console.log('Web socket closed')
      logs.classList.add('connecting')
      document.querySelector('#command').disabled = true
      appendLog(' --- < Lost Connection, attempting to reconnect... > ---')
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

  window.onload = function(){
    connect();
  }
}

export default function Home() {

  useEffect(() => {
    if (typeof window !== 'undefined') {
      browser()
    }
  }, [])

  return (
    <div className={styles.container}>

      <Head>
        <title>HOPR Admin</title>
      </Head>

      <div className='logo'>
        <Logo />
      </div>

      <h1>HOPR Logs [TESTNET NODE]</h1>

      <div className='logs connecting'>
        <pre>
          <code id='log'>Connecting...</code>
        </pre>
      </div>

      <div className='send'>
        <input id="command"
          type="text"
          disabled={true}
          placeholder="type 'help' for full list of commands" /> 
      </div>
    </div>
  )
}
