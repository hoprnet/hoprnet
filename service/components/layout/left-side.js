import React, { useState, useEffect } from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import "../../styles/main.scss";

const useUser = () => ({ user: null, loading: false });

const LeftSide = ({ activaMenu }) => {
  const router = useRouter();
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
    <section className="area-left-desktop">
      <div className="menu-desktop">
        <Link href="/">
          <div
            className={
              "menu-item-desktop " + [router.pathname == "/" ? "active" : ""]
            }
          >
            <img src="/assets/icons/home.svg" alt="hopr HOME" />
            <p>HOME</p>
          </div>
        </Link>
        <Link href="/top-assets">
          <div
            className={
              "menu-item-desktop " +
              [router.pathname == "/top-assets" ? "active" : ""]
            }
          >
            <img src="/assets/icons/top.svg" alt="hopr Top ASSETS" />
            <p>TOP ASSETS</p>
          </div>
        </Link>
        <Link href="https://discord.com/invite/wUSYqpD" target="_blank">
          <div className="menu-item-desktop ">
            <img src="/assets/icons/discord.svg" alt="hopr DISCORD" />
            <p>DISCORD</p>
          </div>
        </Link>
        <Link href="/help">
          <div
            className={
              "menu-item-desktop " +
              [router.pathname == "/help" ? "active" : ""]
            }
          >
            <img src="/assets/icons/help.svg" alt="hopr HELP" />
            <p>HELP</p>
          </div>
        </Link>
      </div>
      {/*  */}

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
    </section>
  );
};

export default LeftSide;
