import React, { useState, useEffect, ref } from "react";
import Link from "next/link";
import "../../styles/main.scss";

const Menu = ({ activaMenu }) => {
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
      <div className={"menu-mobile " + [activaMenu ? "open-menu" : ""]}>
        <div className="menu-container">
          <div>
            <ul>
              <Link href="/">
                <li className="active">
                  <img src="/assets/icons/home.svg" alt="hopr HOME" />
                  <p>HOME</p>
                </li>
              </Link>
              <Link href="/top-assets">
                <li>
                  <img src="/assets/icons/top.svg" alt="hopr Top ASSETS" />
                  <p>TOP ASSETS</p>
                </li>
              </Link>
              <Link href="https://discord.com/invite/wUSYqpD">
                <li>
                  <img src="/assets/icons/discord.svg" alt="hopr DISCORD" />
                  <p>DISCORD</p>
                </li>
              </Link>
              <Link href="/help">
                <li>
                  <img src="/assets/icons/help.svg" alt="hopr HELP" />
                  <p>HELP</p>
                </li>
              </Link>
            </ul>

            <hr />
            <div className="quick-code">
              <p>HOPR node</p>
              <div className="hash">
                <p>{hash}</p>
                <div onClick={() => copyCodeToClipboard()}>
                  <img src="/assets/icons/copy.svg" alt="copy" />
                </div>
              </div>
            </div>
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
        {/*  */}
      </div>
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
};

export default Menu;
