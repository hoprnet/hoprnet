import React, { useState, useEffect } from "react";
import DataBoxCloud from "../data-view/data-box-cloud";
import DataUpdateKnow from "../data-view/data-update-know";
import "../../styles/main.scss";

const RightSide = () => {
  return (
    <section className="right-side only-desktop-view">
      <p className="paragraph">
        Welcome to <span>HOPR Basòdino testnet!</span> Visit HELP in the menu
        for instructions. Registration is open. CoverBot will begin relaying
        data on <span>21st Oct</span> at <span>3pm CET</span>. The{" "}
        <span>200</span> highest scorers will win a share of{" "}
        <span>200,000 HOPR</span>. The testnet will run until{" "}
        <span>Nov 4th</span>.{" "}
        <a  
        className="aux-link-out"
          target="_blank"
          href="https://medium.com/hoprnet"
          rel="noopener noreferrer">
          
           follow us on <span>[<img src="/assets/icons/medium.svg" alt="medium" />] medium.</span>
        </a>
      </p>
      <hr />
      <DataBoxCloud />
      <hr />
      <DataUpdateKnow />
    </section>
  );
};

export default RightSide;
