import React, { useState, useEffect } from "react";
import { useRouter } from "next/router";
import Head from "next/head";
import Menu from "../menu/menu";
import LeftSide from "./left-side";
import RightSide from "./right-side";
import CopieParagraph from "../data-view/copie-paragraph";
import DataBoxCloud from "../data-view/data-box-cloud";
import DataUpdateKnow from "../data-view/data-update-know";
import "../../styles/main.scss";

const Layout = ({ children }) => {
  const router = useRouter();
  const [activaMenu, setactivaMenu] = useState(false);

  return (
    <>
      <Head>
        <title>hopr</title>
      </Head>
      <header>
        <nav className="navbar only-mobile-view">
          <div className="icon-logo">
            <img
              className={[activaMenu ? "open" : ""]}
              src="/assets/brand/logo.svg"
              alt="hopr"
            />
          </div>
          <div
            className={"icon-menu " + [activaMenu ? "open" : ""]}
            onClick={() => setactivaMenu(!activaMenu)}
          >
            <span></span>
            <span></span>
            <span></span>
            <span></span>
            <span></span>
            <span></span>
          </div>
        </nav>
        {/*  */}
        <div className=" only-desktop-view ">
          <div className="icon-logo-desktop">
            <a href="https://hoprnet.org/" target="_blank"  rel="noopener noreferrer">
              <img src="/assets/brand/logo.svg" alt="hopr" />
            </a>
          </div>
        </div>
      </header>
      <Menu activaMenu={activaMenu} />

      <div className="main-container">
        <div className="only-desktop-view">
          <LeftSide />
        </div>
        <section
          className={
            "about only-mobile-view " +
            [router.pathname != "/" ? "aux-margin" : ""]
          }
        >
          <div className={[router.pathname != "/" ? "only-desktop-view" : ""]}>
           <CopieParagraph/>
          </div>
        </section>
        {children}
        {/*  */}
        <section className="only-mobile-view">
          <hr />
          <DataBoxCloud />
          <hr />
          <DataUpdateKnow />
          <hr />
          <p className="paragraph-copy ">
            Thanks for helping us create the <span> HOPR network. </span>
          </p>
        </section>
        {/*  */}
        <RightSide />
        {/*  */}
      </div>
    </>
  );
};

export default Layout;
