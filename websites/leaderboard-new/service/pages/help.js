import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import TweetBasodino from "../components/tweet-basodino";
import api from "../utils/api";

export default function Help() {
  const [hash, setHash] = useState(
    "16Uiu2HAm7KxaBkgd9ENvhf5qAkp1c6Q5Q1dXe8HBDzxLN4SxAVw6"
  );

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) setHash(response.data.address);
    };
    fetchData();
  }, []);

  const [modal, setModal] = useState(false);
  const copyCodeToClipboard = () => {
    navigator.clipboard.writeText(hash);
    setModal(true);
    setTimeout(() => {
      setModal(false);
    }, 4000);
  };

  return (
    <>
      <Layout>
        <div className="box">
          <div className="box-top-area">
            <div>
              <div className="box-title">
                <h1>Instructions</h1>
              </div>
              <div className="box-btn">
                <p>v0.05</p>
              </div>
            </div>
          </div>

          <div className="box-main-area">
            <hr />
            <ol>
              <li>
                Install the latest version of{" "}
                <a
                  href="https://github.com/hoprnet/hopr-chat/releases"
                  target="_blank"
                  rel="noreferrer"
                >
                  HOPR Chat
                </a>
                , which will spin up a HOPR node.
              </li>
              <li>
                Send <strong>0.02 Matic</strong> to your node. You can get Matic
                from ETH on{" "}
                <a
                  href="//wallet.matic.network"
                  target="_blank"
                  rel="noreferrer"
                >
                  wallet.matic.network
                </a>{" "}
                or ping us on{" "}
                <a href="//t.me/hoprnet" target="_blank" rel="noreferrer">
                  Telegram
                </a>
                .
              </li>
              <li>
                In your HOPR node, type <strong>myAddress</strong> to find your
                node address.
              </li>
              <li>
                Tweet your HOPR node address with the tag{" "}
                <strong>#Basodino</strong> and <strong>@hoprnet</strong>.{" "}
                <TweetBasodino>
                  <img src="/assets/icons/twitter.svg" alt="twitter" />{" "}
                  #Basodino
                </TweetBasodino>
              </li>
              <li>
                In your HOPR node, type{" "}
                <strong>settings includeRecipient true</strong> so the bot can
                respond.
              </li>
              <li>
                Send the URL of your tweet to the <strong>CoverBot</strong>{" "}
                using the <strong>send</strong> command. You may need to use{" "}
                <strong>crawl</strong> first.
                <br />
                <div className="quick-code">
                  <div className="hash" onClick={() => copyCodeToClipboard()}>
                    <p>{hash}</p>
                    <div>
                      <img src="/assets/icons/copy.svg" alt="copy" />
                    </div>
                  </div>
                </div>
              </li>
              <li>Wait for a message from CoverBot verifying your tweet.</li>
              <li>
                You have scored points! Keep your node online to earn more!
              </li>
              <li>
                Every 30s, CoverBot will randomly choose a registered user to
                relay data and earn more points.
              </li>
            </ol>
            <hr />
            <div className="twitter-line-menu">
              <div>
                <a href="https://twitter.com/hoprnet" target="_blank">
                  <img src="/assets/icons/twitter.svg" alt="twitter" />
                  <p>@hoprnet</p>
                </a>
              </div>
              <div>
                <TweetBasodino>
                  <img src="/assets/icons/twitter.svg" alt="twitter" />{" "}
                  <p>#Basodino</p>
                </TweetBasodino>
              </div>
            </div>
          </div>
        </div>
      </Layout>
      <div className={"modal-copy-menu " + [modal ? "show-modal-menu" : ""]}>
        <div className="box-modal-copy">
          <div className="icon-logo">
            <img src="/assets/brand/logo.svg" alt="hopr" />
          </div>
          <div className="content">
            <div>
              <p>{hash}</p>
            </div>
            <h5>copied to clipboard</h5>
            <hr className="hr-alert" />
            <p className="copy-alert">
              this message is only informative it <br />
              closes itself in <span>4 seconds.</span>
            </p>
          </div>
        </div>
      </div>
    </>
  );
}
