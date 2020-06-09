import Head from 'next/head'
import Faucet from '../components/Faucet'
import { colors, NETWORK } from '../utils'

export default function Home() {
  return (
    <div className="container">
      <Head>
        <title>HOPR token Faucet</title>
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <main>
        <h1 className="title">
          Welcome to the <a href="https://hopr.network/">HOPR</a> token faucet!
        </h1>

        <p className="description">This faucet will mint 100 HOPR tokens to your address.</p>

        <div>
          <Faucet network={NETWORK} />
        </div>
      </main>

      <footer>
        <a href={`http://${NETWORK}.etherscan.io/`} target="_blank" rel="noopener noreferrer">
          {NETWORK}&nbsp;
          <span className="dot" style={{ backgroundColor: colors[NETWORK] }} />
        </a>

        <a href="https://github.com/hoprnet" target="_blank" rel="noopener noreferrer">
          hoprnet <img src="/github-32x.png" alt="Github Logo" className="logo" />
        </a>
      </footer>

      <style jsx>{`
        .container {
          min-height: 100vh;
          padding: 0 0.5rem;
          display: flex;
          flex-direction: column;
          justify-content: center;
          align-items: center;
        }

        main {
          padding: 5rem 0;
          flex: 1;
          display: flex;
          flex-direction: column;
          justify-content: center;
          align-items: center;
        }

        footer {
          width: 100%;
          margin: 0 2em;
          height: 75px;
          border-top: 1px solid #eaeaea;
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        footer img {
          margin-left: 0.5rem;
        }

        footer a {
          display: flex;
          align-items: center;
        }

        footer a:first-of-type {
          flex: 1;
          align-items: left;
        }

        footer::after {
          flex: 1;
          content: '';
        }

        a {
          color: inherit;
          text-decoration: none;
        }

        .title a {
          color: #0070f3;
          text-decoration: none;
        }

        .title a:hover,
        .title a:focus,
        .title a:active {
          text-decoration: underline;
        }

        .title {
          margin: 0;
          line-height: 1.15;
          font-size: 3rem;
        }

        .title,
        .description {
          text-align: center;
        }

        .description {
          line-height: 1.5;
          font-size: 1.5rem;
        }

        .logo {
          height: 1em;
        }

        .dot {
          height: 10px;
          width: 10px;
          background-color: #bbb;
          border-radius: 50%;
          display: inline-block;
        }
      `}</style>

      <style jsx global>{`
        html,
        body {
          padding: 0;
          margin: 0;
          font-family: 'Courier New', Courier, monospace;
        }

        * {
          box-sizing: border-box;
        }
      `}</style>
    </div>
  )
}
