import React, { useState, useEffect } from "react";
import api from '../../utils/api'
import "../../styles/main.scss";

const DataBoxCloud = () => {
  const [API_Available, SetAPI_Available] = useState(null);
  const [API_Balance, SetAPI_Balance] = useState(null);
  const [API_HoprChannelContract, SetAPI_HoprChannelContract] = useState(null);
  const [API_HoprCoverbotAddress, SetAPI_HoprCoverbotAddress] = useState(null);

  useEffect(() => {
    const fetchStats = async () => {
      const apiStats = await api.getState();
      if (apiStats.data) {
        SetAPI_Available(apiStats.data.available);
        SetAPI_Balance(apiStats.data.balance);
        SetAPI_HoprChannelContract(apiStats.data.hoprChannelContract);
        SetAPI_HoprCoverbotAddress(apiStats.data.hoprCoverbotAddress);
      }
    }
    fetchStats();
  }, []);


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
