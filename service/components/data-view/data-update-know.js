import React, { useState, useEffect } from "react";
import Head from "next/head";
import "../../styles/main.scss";

const DataUpdateKnow = ({ children }) => {

  return (
    <div className="box-info">
      <div>
        <p>Channel: <span>0x83cA7023c4B1EDB137E1d87B3D05F</span></p>
      </div>
      <div>
        <p>Coverbot: <span>0x1d157417E639ACA5581D96236089…</span></p>
      </div>
      <div>
        <p>Last Updated: <span>2020-10-02 10:02</span></p>
      </div>
    </div>
  );
};

export default DataUpdateKnow;
