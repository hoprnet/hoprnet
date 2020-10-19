import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import Table from "rc-table";

export default function Home() {
  const [API_CORE, SetAPI_CORE] = useState(null);
  useEffect(() => {
    getData();
  }, []);

  const getData = async () => {
    const data = await fetch(
      "https://hopr-coverbot.firebaseio.com/basodino-develop-1-17-5/state.json"
    );
    const cleanData = await data.json();
    SetAPI_CORE(cleanData.connected);
  };

  const columns = [
    {
      title: "address",
      dataIndex: "address",
      key: "address",
    },
    {
      title: "id",
      dataIndex: "id",
      key: "id",
    },
    {
      title: "tweetId",
      dataIndex: "tweetId",
      key: "tweetId",
    },
    {
      title: "tweetUrl",
      dataIndex: "tweetUrl",
      key: "tweetUrl",
    },
  ];

  return (
    <Layout>
      <div className="box">
        <div className="box-top-area">
          <div>
            <div className="box-title">
              <h1>Leaderboard</h1>
            </div>
            <div className="box-btn">
              <button>
                <img src="/assets/icons/refresh.svg" alt="refresh now" />
                refresh now
              </button>
            </div>
          </div>

          <div className="box-menu-optional">
            <ul>
              <li className="active">All</li>
              <li>Verified</li>
              <li>Registered</li>
              <li>Connected</li>
            </ul>
          </div>
        </div>
        <div className="box-main-area">
          <div className="box-container-table">
            <Table useFixedHeader={true} columns={columns} data={API_CORE} rowKey={(e) => e.id} />
          </div>
        </div>
      </div>
    </Layout>
  );
}
