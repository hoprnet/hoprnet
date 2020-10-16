import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";

export default function Help() {
  const [hash, setHash] = useState(
    "16Uiu2HAmRE4fVtp8dF6H62NzRcx6LGUTL5fBRTdnAfZXjveP5Kz9"
  );

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
            <p>
              Install the latest version of HOPR Chat, which will spin up a HOPR
              node. Send 0.02 xDAI to your node. You can get xDAI from ETH on
              xdai.io or ping us on Telegram. In your HOPR node, type myAddress
              to find your node address. Tweet your HOPR node address with the
              tag #HOPRNetwork and @hoprnet. In your HOPR node, type
              includeRecipient and then “y” so the bot can respond. Send the URL
              of your tweet to the CoverBot using the send command:
            </p>
            <div className="quick-code">
              <div className="hash" onClick={() => copyCodeToClipboard()}>
                <p>{hash}</p>
                <div>
                  <img src="/assets/icons/copy.svg" alt="copy" />
                </div>
              </div>
            </div>
            <p>
              Wait for a message from CoverBot verifying your tweet. You have
              scored points! Keep your node online to earn more! Every 30s,
              CoverBot will randomly choose a registered user to relay data and
              earn more points.
            </p>
            <hr />
            <div className="twitter-line-menu">
              <div>
                <a href="https://twitter.com/hoprnet" target="_blank">
                  <img src="/assets/icons/twitter.svg" alt="twitter" />
                  <p>@hoprnet</p>
                </a>
              </div>
              <div>
                <a
                  href="https://twitter.com/intent/tweet?original_referer=https%3A%2F%2Fsaentis.hoprnet.org%2F&amp;ref_src=twsrc%5Etfw&amp;related=hoprnet&amp;text=Signing%20up%20to%20earn%20%24HOPR%20on%20the%20%23HOPRnetwork.%20My%20%40hoprnet%20address%20is%3A%20&amp;tw_p=tweetbutton"
                  target="_blank"
                >
                  <img src="/assets/icons/twitter.svg" alt="twitter" />
                  <p>#HOPRNetwork</p>
                </a>
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
