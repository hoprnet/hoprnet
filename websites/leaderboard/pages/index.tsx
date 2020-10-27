import Head from 'next/head'
import styles from '../styles/Home.module.css'
import CopyIcon from '../components/icons/copy'
import TwitterIcon from '../components/icons/twitter'
import { useState, useEffect } from 'react'
import useSWR from 'swr'
import api from '../utils/api'
import { default as Footer } from '../components/Footer/Footer'
import { default as Header } from '../components/Header/Header'
import BlockscoutLink from '../components/BlockscoutLink'
import { EnvironmentProps } from '../utils/env'
import { FirebaseScoreMap, FirebaseStateRecords } from '../utils/db'

interface ScoredNodeProps {
  address: string
  score: number
  connected: any[]
}

const ScoredNode: React.FC<ScoredNodeProps> = ({ address, score, connected }: ScoredNodeProps) => {
  const n = connected.find((n) => n.address == address)
  // Find node
  if (n) {
    const twitterHandle = n.tweetUrl.match(/twitter.com\/([^\/]+)\/.*/i)[1]
    return (
      <div className={styles.connode}>
        <span className={styles.score}>{score}</span>
        <BlockscoutLink id={address}>
          <strong>@{twitterHandle}</strong>
        </BlockscoutLink>
        <span className={styles.addr}>
          Node: <abbr title={n.id}>...{n.id.slice(45)}</abbr>
        </span>
        <a rel="noreferrer" target="_blank" href={n.tweetUrl}>
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
  address = '0x000...',
  available = '',
  locked = '',
  connected = [],
  connectedNodes = [],
  hoprChannelContract = '0x000...',
  hoprCoverbotAddress = '0x000...',
  env = {} as EnvironmentProps,
  refreshed = new Date().toString(),
}) {
  const addressOnClick = () => {
    const addressClicker = document.createElement('textarea')
    addressClicker.value = address
    document.body.appendChild(addressClicker)
    addressClicker.select()
    document.execCommand('copy')
    document.body.removeChild(addressClicker)
  }

  const [score, setScore] = useState<FirebaseScoreMap>({})

  useEffect(() => {
    const fetchScore = async () => {
      const apiScoreResponse = await api.getScore()
      if (apiScoreResponse.data) {
        const score: FirebaseScoreMap = apiScoreResponse.data as FirebaseScoreMap
        setScore(score)
      }
    }
    fetchScore()
  }, [])

  const scoreArray = Object.keys(score).map((key) => ({ address: key, score: score[key] }))

  return (
    <>
      <Head>
        <title>HOPR Incentivized Testnet on xDAI</title>
        <script async src="https://platform.twitter.com/widgets.js" charSet="utf-8"></script>
      </Head>

      <Header {...{ available, locked }} />

      <section className={styles.intro}>
        <p>
          Welcome to HOPR Säntis testnet! Follow the instructions below to start earning points. There are HOPR token
          prizes for the 20 highest scorers, along with 10 random prizes. The testnet will run until October 6th.
        </p>
        <br />
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
              Send <strong>{Math.max(parseFloat(env ? `${env.COVERBOT_XDAI_THRESHOLD}` : '0'), 0.02)} xDAI</strong> to
              your{' '}
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
            </li>
            <li>
              In your HOPR node, type <strong>includeRecipient</strong> and then “y” so the bot can respond.
            </li>
            <li>
              Send the URL of your tweet to the{' '}
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
            <li>Every 30s, CoverBot will randomly choose a registered user to relay data and earn more points.</li>
          </ol>
        </section>

        <section>
          <div className={styles.padBottom}>
            <h2>Leaderboard</h2>
            <h3 style={{ paddingLeft: '20px' }}>
              {connected.length} verified | {scoreArray.length} registered | {connectedNodes} connected
            </h3>
            {(scoreArray.length == 0 || connected.length == 0) && (
              <p className={styles.conerr}>
                <em>No nodes connected...</em>
              </p>
            )}
            {scoreArray.length > 0 &&
              scoreArray.map((n) => <ScoredNode key={n.address} {...n} connected={connected} />)}
          </div>
        </section>
      </main>

      <Footer {...{ hoprChannelContract, hoprCoverbotAddress, styles, refreshed }} />
    </>
  )
}

const fetcher = (url: string) => fetch(url).then((res) => res.json())

export async function getServerSideProps() {
  const state = await api.getState()
  return { props: state }
}

const Home: React.FC<FirebaseStateRecords> = (props) => {
  const { data } = useSWR('/api/state', fetcher, {
    initialData: props || {},
    refreshInterval: 5000,
  })
  console.log('Home', data)
  return <HomeContent {...data} />
}

export default Home
