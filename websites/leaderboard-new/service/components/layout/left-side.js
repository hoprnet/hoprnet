import React, { useState, useEffect } from "react";
import { useRouter } from "next/router";
import Link from "next/link";
import "../../styles/main.scss";
import TweetBasodino from "../tweet-basodino";
import api from "../../utils/api";

const LeftSide = () => {
  const router = useRouter();
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
        <Link href="/hopr-allocation">
          <div
            className={
              "menu-item-desktop " +
              [router.pathname == "/hopr-allocation" ? "active" : ""]
            }
          >
            <img src="/assets/icons/top.svg" alt="hopr HOPR ALLOCATION" />
            <p>HOPR <br/> ALLOCATION</p>
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
      <div className="copy-line-token">
        <h4>
        HOPR node
        </h4>
        <div className="hash" onClick={() => copyCodeToClipboard()}>
            <p>{hash}</p>
          <div>
            <img src="/assets/icons/copy.svg" alt="copy" />
          </div>
        </div>
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
