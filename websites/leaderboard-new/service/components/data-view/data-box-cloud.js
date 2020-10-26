import React from "react";
import "../../styles/main.scss";

const DataBoxCloud = ({address, channel}) => {

  const copyCodeToClipboard = (aux) => {
    navigator.clipboard.writeText(aux);
  };
 
  return (
    <div className="box-border">
      <div onClick={() => copyCodeToClipboard(address)}>
        <h3 className="num"> {address} </h3>
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
