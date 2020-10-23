import React, { useState, useEffect } from "react";
import "../../styles/main.scss";

import api from "../../utils/api";

const DataBoxCloud = () => {
  const [Address, setAddress] = useState("");
  const [channel, setChannel] = useState("");

  const copyCodeToClipboard = (aux) => {
    navigator.clipboard.writeText(aux);
  };

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) {
        setAddress(response.data.hoprCoverbotAddress);
        setChannel(response.data.hoprChannelContract);
      }
    };
    fetchData();
  }, []);

  return (
    <div className="box-border">
      <div onClick={() => copyCodeToClipboard(token)}>
        <h3 className="num"> {Address} </h3>
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
