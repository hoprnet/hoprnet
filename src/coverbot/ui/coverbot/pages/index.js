import Head from 'next/head'
import styles from '../styles/Home.module.css'
import Logo from '../components/logo'
import CopyIcon from '../components/icons/copy'
import TwitterIcon from '../components/icons/twitter'
import { useState, useEffect } from 'react'
import useSWR from 'swr'
import db from '../utils/db'
import { HOPR_ENVIRONMENT } from '../utils/env'

function BSLink({ id, children }) {
  return (
    <a target="_blank" href={'https://blockscout.com/poa/xdai/address/' + id + '/transactions'}>
      {children}
    </a>
  )
}

function ConnectedNode({ id, address, tweetUrl }) {
  return (
    <div className={styles.connode}>
      <BSLink id={address}>
        <strong>{id}</strong>
      </BSLink>
      <a target="_blank" href={tweetUrl}>
        <TwitterIcon />
      </a>
    </div>
  )
}

function ScoredNode({ address, score }) {
  return (
    <div className={styles.connode}>
      <strong>{address}</strong>-{score}
    </div>
  )
}

function HomeContent({
  address,
  available,
  locked,
  connected,
  hoprChannelContract,
  hoprCoverbotAddress,
  env,
  refreshed,
}) {
  let addressOnClick = () => {
    let addressClicker = document.createElement('textarea')
    addressClicker.value = address
    document.body.appendChild(addressClicker)
    addressClicker.select()
    document.execCommand('copy')
    document.body.removeChild(addressClicker)
  }

  const [score, setScore] = useState([])

  useEffect(() => {
    db.ref(`/${HOPR_ENVIRONMENT}/score`)
      .orderByValue()
      .on('value', (snapshot) => {
        const result = snapshot.val()
        setScore(
          Object.entries(result || {}).map(([address, score]) => ({
            address,
            score,
          })),
        )
      })
  }, [])

  return (
    <>
      <Head>
        <title>HOPR Incentivized Testnet on xDAI</title>
        <script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script>
      </Head>

      <header className={styles.header}>
        <div className={styles.h1}>
          <Logo />
          <h1 className={styles.title}>
            <a href="https://hoprnet.org">HOPR</a> Incentivized Testnet on xDAI
          </h1>
        </div>

        <div className={styles.stats}>
          <div>
            <strong className="green">{parseFloat(available).toFixed(4)}</strong> xHOPR Available
          </div>
          <div>
            <strong className="blue">{locked}</strong> xHOPR Locked
          </div>
        </div>
      </header>

      <section className={styles.intro}>
          <p>Welcome to HOPR Säntis testnet! Follow the instructions below to start earning points. 
            There are HOPR token prizes for the 20 highest scorers, along with 10 random prizes. 
            The testnet will run until 14th September</p>
          <p>Click <a href="https://docs.hoprnet.org/home/getting-started/saentis-testnet">here</a> for
            more information about the testnet and HOPR. Join our <a href="https://discord.gg/wUSYqpD">Discord</a> for support and feedback.</p>
      </section>
      

      <main className={styles.main}>
        <section className={styles.instructions}>
          <h2>Instructions</h2>
          <ol>
            <li>
              Download <a href="https://github.com/hoprnet/hopr-chat/releases">HOPR Node Säntis</a> and run it.
            </li>
            <li>
              Send <strong>{Math.max(parseFloat(env ? env.COVERBOT_XDAI_THRESHOLD : 0), 0.01)} xDAI</strong> to your
              node
            </li>
            <li>
              In <strong>hopr-chat</strong> enable recipient by using <strong>includeRecipient</strong> command
            </li>
            <li>
              <>
                Tweet your HOPR node address with the tag <strong>#HOPRNetwork</strong> and <strong>@hoprnet</strong>{' '}
                <a
                  href="https://twitter.com/intent/tweet?ref_src=twsrc%5Etfw"
                  className="twitter-hashtag-button"
                  data-text="Signing up to earn $HOPR on the #HOPRnetwork. My @hoprnet address is: "
                  data-related="hoprnet"
                  data-show-count="false"
                >
                  Tweet #hoprnetwork
                </a>
              </>
            </li>
            <li>
              Send a message with your tweet to the Cover Node address using the <strong>send</strong> command:
              <br />
              <strong onClick={addressOnClick} className={styles.address}>
                {address} <CopyIcon />
              </strong>
            </li>
            <li>Wait for the Cover bot to send you a message</li>
            <li>You have received xHOPR tokens!</li>
          </ol>
        </section>

        <section>
          <div className={styles.padBottom}>
            <h2>Leaderboard</h2>
            {score.length == 0 && (
              <p className={styles.conerr}>
                <em>No nodes scored...</em>
              </p>
            )}
            {score.length > 0 && score.map((n) => <ScoredNode {...n} />)}
          </div>


          <div>
            <h2>Connected HOPR nodes</h2>
            {connected.length == 0 && (
              <p className={styles.conerr}>
                <em>No nodes connected...</em>
              </p>
            )}
            {connected.length > 0 && connected.map((n) => <ConnectedNode {...n} />)}
          </div>
        </section>
      </main>

      <footer className={styles.footer}>
        <div>
          <div>
            <BSLink id={hoprChannelContract}>
              <strong>Channel:</strong>
              {hoprChannelContract && hoprChannelContract.slice(0, 8)}...
            </BSLink>
          </div>
          <div>
            <BSLink id={hoprCoverbotAddress}>
              <strong>Coverbot:</strong>
              {hoprCoverbotAddress && hoprCoverbotAddress.slice(0, 8)}...
            </BSLink>
          </div>
        </div>

        Thanks for helping us create the <a href="https://hoprnet.org/">HOPR</a> network.
        <br />
        <br />
        Updated: {refreshed}
      </footer>
    </>
  )
}

const fetcher = (...args) => fetch(...args).then((res) => res.json())

export async function getServerSideProps() {
  let api = require('./api/stats')
  return { props: api.get() } // NextJS makes this stupidly complicated
}

export default function Home(props) {
  let { data, error } = useSWR('/api/stats', fetcher, { initialData: props || null, refreshInterval: 5000 })
  if (!data || !Object.keys(data).length) {
    // SWR inits to {} with initalData = undefined :(
    return <div>...</div>
  }
  return <HomeContent {...data} />
}
