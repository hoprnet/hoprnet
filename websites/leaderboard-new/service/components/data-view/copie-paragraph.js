import React, { useState, useEffect } from "react";
import "../../styles/main.scss";

const CopieParagraph = () => {
  return (
    <>
      <p className="paragraph">
        Welcome to{" "}
        <a
          className="aux-link-out-to-post"
          target="_blank"
          href="https://medium.com/hoprnet/hopr-bas%C3%B2dino-a-better-bigger-braver-testnet-97f68e1c9b7e"
          rel="noopener noreferrer"
        >
          <span>HOPR Basòdino testnet!</span>{" "}
        </a>
        Visit HELP in the menu for instructions. Registration is open. CoverBot
        will begin relaying data on <span>21st Oct</span> at{" "}
        <span>3pm CET</span>. The <span>200</span> highest scorers will win a
        share of <span>200,000 HOPR</span>. The testnet will run until{" "}
        <span>Nov 4th</span>.{" "}
        <a
          className="aux-link-out"
          target="_blank"
          href="https://medium.com/hoprnet"
          rel="noopener noreferrer"
        >
          Follow us on{" "}
          <span>
            [<img src="/assets/icons/medium.svg" alt="medium" />] medium.
          </span>
        </a>
      </p>
    </>
  );
};

export default CopieParagraph;
