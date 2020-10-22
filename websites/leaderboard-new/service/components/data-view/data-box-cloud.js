import React, { useState, useEffect } from "react";
import api from '../../utils/api'
import "../../styles/main.scss";

const DataBoxCloud = () => {


  useEffect(() => {
    const fetchStats = async () => {
      const apiStats = await api.getState();
   
    }
    fetchStats();
  }, []);


  return (
    <div className="box-border">
      <div>
        <h3 className="num"> 0x4774fEd3f2838f504006BE53155cA9cbDDEe9f0c </h3>
        <p>HOPR token address</p>
      </div>
     
      <div>
        <h3 className="num"> 0x25E2e5D8EcC4fE46a9505079Ed29266779dC7D6f </h3>
        <p>HOPR payment channe</p>
      </div>
    </div>
  );
};

export default DataBoxCloud;
