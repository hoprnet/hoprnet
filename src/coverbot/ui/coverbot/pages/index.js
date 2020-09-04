import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Logo from '../components/logo'
import {useRef} from 'react'

function ConnectedNode({id, locked, claimed}){
  return (
    <div className={styles.connode}>
      <strong>{ id }</strong>
      <span className='blue'>{locked}</span>
      <span className='red'>{claimed}</span>
    </div>
  )
}

export async function getServerSideProps() {
  return {
    props: {
      address: '0x1234567891012',
      available: 123.00,
      locked: 2.00,
      claimed: 123.00,
      connected: [
        {id: '0x12345', locked: 12, claimed: 0},
        {id: '0x32345', locked: 42, claimed: 7},
      ]
    }
  }
}

export default function Home({
  address,
  available,
  locked,
  claimed,
  connected
}) {

  let addressOnClick = () => {
    let addressClicker = document.createElement('textarea')
    addressClicker.value = address
    document.body.appendChild(addressClicker)
    addressClicker.select()
    document.execCommand('copy')
    document.body.removeChild(addressClicker)
  }

  return (
    <div className={styles.container}>
      <Head>
        <title>HOPR Incentivized Testnet on xDAI</title>
      </Head>


      <header className={styles.header}>
        <Logo />
        <h1 className={styles.title}>
          HOPR Incentivized Testnet on xDAI
        </h1>
        <div className={styles.stats}>
          <h2>xHOPR Tokens</h2>
          <div>
            <span className='green'>{available}</span> Available
          </div>
          <div>
            <span className='blue'>{locked}</span> Locked
          </div>
          <div>
            <span className='red'>{claimed}</span> Claimed
          </div>
        </div>
      </header>

      <main className={styles.main}>
        <section className={styles.instructions}>
          <h2>Instructions</h2>
          <ol>
            <li>Download <a href="#">HOPR Node Saentis</a> and run it.</li>
            <li>Send <strong>10 xDAI</strong> to your node</li>
            <li><a href="#">Tweet</a> your HOPR node address with the tag <strong>#HOPRNetwork</strong> and <strong>@hoprnet</strong></li>
            <li>Send a message with your tweet to the Cover Node address:
              <br />
              <strong onClick={addressOnClick} className={styles.address}>
                { address }
              </strong>
            </li>
            <li>Wait for the Cover bot to send you a message</li>
            <li>You have received xHOPR tokens!</li>
          </ol>
        </section>

        <section>
          <h2>Connected HOPR nodes</h2>
          { connected.length == 0 && 
            <p className={styles.conerr}>
              <em>No nodes connected...</em>
            </p>
          }
          { connected.length > 0 &&
            connected.map(n => <ConnectedNode {...n} />)
          }
        </section>
      </main>

      <footer className={styles.footer}>
      </footer>
    </div>
  )
}
