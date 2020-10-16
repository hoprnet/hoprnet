import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";

export default function Help() {
    const [hash, setHash] = useState(
        "16Uiu2HAmRE4fVtp8dF6H62NzRcx6LGUTL5fBRTdnAfZXjveP5Kz9"
      );

      const copyCodeToClipboard = () => {
        navigator.clipboard.writeText(hash);
    
      };
  return (
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
            xdai.io or ping us on Telegram. In your HOPR node, type myAddress to
            find your node address. Tweet your HOPR node address with the tag
            #HOPRNetwork and @hoprnet. In your HOPR node, type includeRecipient
            and then “y” so the bot can respond. Send the URL of your tweet to
            the CoverBot using the send command:
          </p>
          <div className="quick-code">
              <div className="hash">
                <p>{hash}</p>
                <div onClick={() => copyCodeToClipboard()}>
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
              <a href="#" target="_blank">
                <img src="/assets/icons/twitter.svg" alt="twitter" />
                <p>@hoprnet</p>
              </a>
            </div>
            <div>
              <a href="#" target="_blank">
                <img src="/assets/icons/twitter.svg" alt="twitter" />
                <p>#HOPRNetwork</p>
              </a>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
