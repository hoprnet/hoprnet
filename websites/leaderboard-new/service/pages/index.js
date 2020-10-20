import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import api from "../utils/api";

export default function Home() {
  const [data, setData] = useState(undefined);
  console.log("data", data);
  const nodesVerified = data ? data.connected.length : 0;
  const nodesRegistered = data ? data.nodes.length : 0;
  const nodesConnected = data ? data.connectedNodes : 0;
  const nodes = data ? data.nodes : [];

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) setData(response.data);
    };
    fetchData();
  }, []);

  const columns = [
    {
      title: "online",
      dataIndex: "online",
      key: "online",
    },
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
      title: "score",
      dataIndex: "score",
      key: "score",
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
              <li>
                {nodesVerified && <span>{nodesVerified}</span>}
                Verified
              </li>
              <li>
                {nodesRegistered && <span>{nodesRegistered}</span>}
                Registered
              </li>
              <li>
                {nodesConnected && <span>{nodesConnected}</span>}
                Connected
              </li>
            </ul>
          </div>
        </div>
        <div className="box-main-area remove-all-padding">
          <div className="box-container-table">
            {nodes && (
              <table id="date">
                <thead>
                  <tr>
                    {columns.map((e) => {
                      const { title, key } = e;
                      return (
                        <th scope="col" key={key}>
                          {title}
                        </th>
                      );
                    })}
                  </tr>
                </thead>
                <tbody>
                  {nodes.map((e) => {
                    const { online, address, id, score, tweetId, tweetUrl } = e;
                    return (
                      <tr key={id}>
                        <td data-label="online">{online ? "🟢" : "🔴"}</td>
                        <td data-label="address">{address}</td>
                        <td data-label="id">{id}</td>
                        <td data-type="score" data-label="score">
                          {score}
                        </td>
                        <td data-label="tweetId">{tweetId}</td>
                        <td data-label="tweetUrl">
                          <a href={tweetUrl}>
                            <img
                              src="/assets/icons/twitter.svg"
                              alt="twitter"
                            />
                          </a>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            )}
          </div>
        </div>
      </div>
    </Layout>
  );
}
