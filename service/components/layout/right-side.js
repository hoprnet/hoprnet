import React, { useState, useEffect } from "react";
import DataBoxCloud from "../data-view/data-box-cloud";
import DataUpdateKnow from "../data-view/data-update-know";
import "../../styles/main.scss";

const RightSide = () => {
  return (
    <section className="right-side only-desktop-view">
      <p className="paragraph">
        Welcome to <span>HOPR Basòdino testnet!</span> Click HELP on the left
        for instructions. Registration is open. CoverBot will begin relaying
        data on 21st Oct at 3pm CET. The <span>200</span> highest scorers will
        win a share of <span>200,000 HOPR</span>. The testnet will run until{" "}
        <span>Nov 4th</span>.
      </p>
      <hr />
      <DataBoxCloud />
      <hr />
      <DataUpdateKnow />
    </section>
  );
};

export default RightSide;
