import React, { useState, useEffect } from "react";
import "../../styles/main.scss";

const DataBoxCloud = () => {
  const [token, setToken] = useState(
    "0x4774fEd3f2838f504006BE53155cA9cbDDEe9f0c"
  );
  const [channel, setChannel] = useState(
    "0x25E2e5D8EcC4fE46a9505079Ed29266779dC7D6f"
  );

  const copyCodeToClipboard = (aux) => {
     navigator.clipboard.writeText(aux);
    // setModal(true);
    // setTimeout(() => {
    //   setModal(false);
    // }, 4000);
  };

  return (
    <div className="box-border">
      <div onClick={() => copyCodeToClipboard(token)}>
        <h3 className="num"> {token} </h3>
        <p>HOPR token address</p>
      </div>
      <div onClick={() => copyCodeToClipboard(channel)}>
        <h3 className="num"> {channel} </h3>
        <p>HOPR payment channel</p>
      </div>
    </div>
  );
};

export default DataBoxCloud;
