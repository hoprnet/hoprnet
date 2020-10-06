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

function ScoredNode({ address, score, connected }) {
  var n = connected.find((n) => n.address == address)
  // Find node
  if (n) {
    var twitterHandle = n.tweetUrl.match(/twitter.com\/([^\/]+)\/.*/i)[1]
    return (
      <div className={styles.connode}>
        <span className={styles.score}>{score}</span>
        <BSLink id={address}>
          <strong>@{twitterHandle}</strong>
        </BSLink>
        <span className={styles.addr}>
          Node: <abbr title={n.id}>...{n.id.slice(45)}</abbr>
        </span>
        <a target="_blank" href={n.tweetUrl}>
          <TwitterIcon />
        </a>
      </div>
    )
  } else {
    // Couldn't find in node list.
    return (
      <div className={styles.connode}>
        <span>(Unable to Relay)</span>
        <span className={styles.score}>{score}</span>
        <span className={styles.addr}>
          Node: <abbr title={address}>...{address.slice(10)}</abbr>
        </span>
      </div>
    )
  }
}

function HomeContent({
  address,
  available,
  locked,
  connected,
  connectedNodes,
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
        <script async src="https://platform.twitter.com/widgets.js" charSet="utf-8"></script>
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
          {/* <div>
            <strong className="blue">{locked}</strong> xHOPR Locked
          </div> */}
        </div>
      </header>

      <section className={styles.intro}>
        <p>
          Welcome to HOPR Säntis testnet! Follow the instructions below to start earning points. There are HOPR token
          prizes for the 20 highest scorers, along with 10 random prizes. The testnet will run until October 6th
        </p>
        <p>
          Click <a href="https://docs.hoprnet.org/home/getting-started/saentis-testnet">here</a> for more information
          about the testnet and HOPR. Join our <a href="https://discord.gg/wUSYqpD">Discord</a> for support and
          feedback.
        </p>
      </section>

      <main className={styles.main}>
        <section className={styles.instructions}>
          <h2>Instructions</h2>
          <ol>
            <li>
              Install the latest version of{' '}
              <a href="https://docs.hoprnet.org/home/getting-started/saentis-testnet/quickstart">HOPR Chat</a>, which
              will spin up a HOPR node.
            </li>
            <li>
              Send <strong>{Math.max(parseFloat(env ? env.COVERBOT_XDAI_THRESHOLD : 0), 0.02)} xDAI</strong> to your{' '}
              <a
                href="https://docs.hoprnet.org/home/getting-started/saentis-testnet/funding-your-node"
                target="_blank"
                rel="noreferrer"
              >
                node
              </a>
              . You can get xDAI from ETH on{' '}
              <a href="//xdai.io" target="_blank" rel="noreferrer">
                xdai.io
              </a>{' '}
              or ping us on{' '}
              <a href="t.me/hoprnet" target="_blank" rel="noreferrer">
                Telegram
              </a>
              .
            </li>
            <li>
              In your HOPR node, type <strong>myAddress</strong> to find your node address.
            </li>
            <li>
              <>
                Tweet your HOPR node address with the tag <strong>#HOPRNetwork</strong> and <strong>@hoprnet</strong>.{' '}
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
              In your HOPR node, type <strong>includeRecipient</strong> and then "y" so the bot can respond.
            </li>
            <li>
              Send a message with your tweet to the{' '}
              <a
                href="https://docs.hoprnet.org/home/getting-started/saentis-testnet/coverbot"
                target="_blank"
                rel="noreferrer"
              >
                CoverBot
              </a>{' '}
              using the <strong>send</strong> command:
              <br />
              <strong onClick={addressOnClick} className={styles.address}>
                {address} <CopyIcon />
              </strong>
            </li>
            <li>Wait for a message from CoverBot verifying your tweet.</li>
            <li>You have scored points! Keep your node online to earn more!</li>
          </ol>
        </section>

        <section>
          <div className={styles.padBottom}>
            <h2>Leaderboard</h2>
            <h3 style={{ paddingLeft: '20px' }}>
              {connected.length} verified | {score.length} registered | {connectedNodes} connected
            </h3>
            {(score.length == 0 || connected.length == 0) && (
              <p className={styles.conerr}>
                <em>No nodes connected...</em>
              </p>
            )}
            {score.length > 0 && score.map((n) => <ScoredNode key={n.address} {...n} connected={connected} />)}
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
        Last Updated: {refreshed}
        <script src="https://panther.hoprnet.org/script.js" site="LCFGMVKB" defer></script>
      </footer>
    </>
  )
}

export async function getServerSideProps() {
  let api = require('./api/stats')
  const props = await api.get()
  console.log('API', props)
  return { props } // NextJS makes this stupidly complicated
}

export default function Home(props) {
  return <HomeContent {...props} />
}
