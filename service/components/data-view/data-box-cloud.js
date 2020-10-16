import React, { useState, useEffect } from "react";
import Head from "next/head";
import "../../styles/main.scss";

const DataBoxCloud = ({ children }) => {
  const [activaMenu, setactivaMenu] = useState(false);

  return (
    <div className="box-border">
      <div>
        <h3 className="num">993.334223</h3>
        <p>xHOPR Available</p>
      </div>
      <div>
        <h3 className="num">993.334223</h3>
        <p>Payout Time</p>
      </div>
      <div>
        <h3 className="num">993.334223</h3>
        <p>xHOPR Sent</p>
      </div>
      <div>
        <h3 className="num">993.334223</h3>
        <p>Payout Value</p>
      </div>
    </div>
  );
};

export default DataBoxCloud;
