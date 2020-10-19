import React, { useState, useEffect } from "react";
import DataBoxCloud from "../data-view/data-box-cloud";
import DataUpdateKnow from "../data-view/data-update-know";
import "../../styles/main.scss";

const RightSide = () => {
  return (
    <section className="right-side only-desktop-view">
      <p className="paragraph">
        Welcome to <span>HOPR Säntis testnet!</span> Follow the instructions
        below to start earning points. There are <span>HOPR token</span> prizes
        for the <span>20</span> highest scorers, along with <span>10</span>{" "}
        random prizes. The testnet will run until
        <span>October 6th.</span>
      </p>
      <hr />
      <DataBoxCloud />
      <hr />
      <DataUpdateKnow />
    </section>
  );
};

export default RightSide;
