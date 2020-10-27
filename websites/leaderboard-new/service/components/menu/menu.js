import React, { useState, useEffect } from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import "../../styles/main.scss";
import TweetBasodino from "../tweet-basodino";

const Menu = ({ activaMenu, hash, copyCodeToClipboard}) => {
  const router = useRouter();
  return (
    <>
      <div className={"menu-mobile " + [activaMenu ? "open-menu" : ""]}>
        <div className="menu-container">
          <div>
            <ul>
              <Link href="/">
                <li className={[router.pathname == "/" ? "active" : ""]}>
                  <img src="/assets/icons/home.svg" alt="hopr HOME" />
                  <p>HOME</p>
                </li>
              </Link>
              <Link href="/hopr-allocation">
                <li
                  className={[
                    router.pathname == "/hopr-allocation" ? "active" : "",
                  ]}
                >
                  <img
                    src="/assets/icons/horp_icon.svg"
                    alt="hopr HOPR ALLOCATION"
                  />
                  <p>HOPR ALLOCATION</p>
                </li>
              </Link>
              <Link href="https://discord.com/invite/wUSYqpD" target="_blank">
                <li>
                  <img src="/assets/icons/discord.svg" alt="hopr DISCORD" />
                  <p>DISCORD</p>
                </li>
              </Link>
              <Link href="/help">
                <li className={[router.pathname == "/help" ? "active" : ""]}>
                  <img src="/assets/icons/help.svg" alt="hopr HELP" />
                  <p>HELP</p>
                </li>
              </Link>
            </ul>
            <hr />
            <div className="quick-code">
              <p>HOPR node</p>
              <div className="hash" onClick={() => copyCodeToClipboard()}>
                <p>{hash.slice(0, 8)}<span>...</span>{hash.slice(-8)}</p>
                <div>
                  <img src="/assets/icons/copy.svg" alt="copy" />
                </div>
              </div>
            </div>
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
      </div>   
    </>
  );
};

export default Menu;
