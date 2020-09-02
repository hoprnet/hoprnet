import React, { useEffect, useState } from "react";
import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Logo from '../components/logo'
import { Logs } from '../components/log'
import { Connection } from '../connection'


export default function Home() {
  let connection

  const [connecting, setConnecting] = useState(true);
  const [messages, setMessages] = useState([]); // The fetish for immutability in react means this will be slower than a mutable array..
  const [peers, setConnectedPeers] = useState([]);

  useEffect(() => {
    if (typeof window !== 'undefined') {
      connection = new Connection(setConnecting, setMessages, setConnectedPeers)
      return Connection.disconnect
    }
  }, [])

  return (
    <div className={styles.container}>

      <Head>
        <title>HOPR Admin</title>
      </Head>
      <Logo />
      <h1>HOPR Logs [TESTNET NODE]</h1>

      <Logs messages={messages} connecting={connecting} />

      <div className='send'>
        <input id="command"
          type="text"
          disabled={connecting}
          autoFocus
          placeholder="type 'help' for full list of commands" /> 
      </div>
    </div>
  )
}
