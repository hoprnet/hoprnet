import React from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import "../../styles/main.scss";
import TweetBasodino from "../tweet-basodino";

const LeftSide = () => {
  const router = useRouter();

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

        <div className="menu-item-desktop ">
          <a
            target="_blank"
            href="https://discord.com/invite/wUSYqpD"
            rel="noopener noreferrer"
          >
            <img src="/assets/icons/discord.svg" alt="hopr DISCORD" />
            <p>DISCORD</p>
          </a>
        </div>

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
          <TweetBasodino>
            <img src="/assets/icons/twitter.svg" alt="twitter" />
            <p>#Basodino</p>
          </TweetBasodino>
        </div>
      </div>
    </section>
  );
};

export default LeftSide;
