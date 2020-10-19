import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import api from '../utils/api'
import Table from "rc-table";

export default function Home() {
  const [data, setData] = useState({})

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) {
        console.log('All data: ', response.data);
        setData(response.data);
      }
    }
    fetchData();
  }, []);

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
            {/* <Table useFixedHeader={true} columns={columns} data={score} rowKey={(e) => e.id} /> */}
          </div>
        </div>
      </div>
    </Layout>
  );
}
