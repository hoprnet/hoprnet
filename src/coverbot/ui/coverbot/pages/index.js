import Head from 'next/head'
import styles from '../styles/Home.module.css'

export default function Home() {
  return (
    <div className={styles.container}>
      <Head>
        <title>HOPR Incentivized Testnet on xDAI</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <main className={styles.main}>
        <h1 className={styles.title}>
          HOPR Incentivized Testnet on xDAI
        </h1>

        <h2>Instructions</h2>
        <ol>
          <li>Download <a href="#">HOPR Node Saentis</a> and run it.</li>
          <li>Send 10 xDAI to your node</li>
          <li><a href="#">Tweet</a> your HOPR node address with the tag #HOPRNetwork and @hoprnet</li>
          <li>Send a message with your tweet to the Cover Node address</li>
          <li>Wait for the Cover bot to send you a message</li>
          <li>You have received xHOPR tokens</li>
        </ol>

      </main>

      <footer className={styles.footer}>
      </footer>
    </div>
  )
}
