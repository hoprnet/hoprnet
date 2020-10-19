import React, { useState, useEffect } from "react";
import Head from "next/head";
import "../../styles/main.scss";

const DataBoxCloud = ({ children }) => {
  const [activaMenu, setactivaMenu] = useState(false);
  const [API_Available, SetAPI_Available] = useState(null);
  const [API_Balance, SetAPI_Balance] = useState(null);
  const [API_HoprChannelContract, SetAPI_HoprChannelContract] = useState(null);
  const [API_HoprCoverbotAddress, SetAPI_HoprCoverbotAddress] = useState(null);

  useEffect(() => {
    getData();
  }, []);

  const getData = async () => {
    const data = await fetch(
      "https://hopr-coverbot.firebaseio.com/basodino-develop-1-17-5/state.json"
    );
    const cleanData = await data.json();
    SetAPI_Available(cleanData.available);
    SetAPI_Balance(cleanData.balance);
    SetAPI_HoprChannelContract(cleanData.hoprChannelContract);
    SetAPI_HoprCoverbotAddress(cleanData.hoprCoverbotAddress);
  };

  return (
    <div className="box-border">
      <div>
        <h3 className="num"> {API_Available}</h3>
        <p>Available</p>
      </div>
     
      <div>
        <h3 className="num"> {API_Balance}</h3>
        <p>Balance</p>
      </div>
      <div>
        <h3 className="num"> {API_HoprChannelContract}</h3>
        <p>Hopr Channel Contract</p>
      </div>
      <div>
        <h3 className="num">{API_HoprCoverbotAddress}</h3>
        <p>Hopr Coverbot Address</p>
      </div>
    </div>
  );
};

export default DataBoxCloud;
