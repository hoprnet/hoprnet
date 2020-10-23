import React, { useState, useEffect } from "react";
import "../../styles/main.scss";

const BoxRemember = () => {
  return (
    <div className="area-remember">
      <div>
        <p>If you encountered any issues please let us know on</p>
        <div className="area-links-remember">
          <a
            target="_blank"
            href="https://discord.com/invite/wUSYqpD"
            rel="noopener noreferrer"
          >
            <img src="/assets/icons/discord.svg" alt="hopr DISCORD" />
            <span>Discord</span>
          </a>

          <a href="//t.me/hoprnet" target="_blank" rel="noreferrer">
            <img src="/assets/icons/telegram.svg" alt="hopr telgram" />
            <span>Telegram</span>
          </a>
        </div>
      </div>
    </div>
  );
};

export default BoxRemember;
