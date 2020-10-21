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
        <h3 className="num"> coming soon </h3>
        <p>HOPR token address</p>
      </div>
     
      <div>
        <h3 className="num"> coming soon </h3>
        <p>HOPR payment channe</p>
      </div>
    </div>
  );
};

export default DataBoxCloud;
