import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import api from "../utils/api";

export default function Home() {
  const [data, setData] = useState({});
  const [dataTable, setDataTable] = useState(false);
  const [dataConnectedNodes, setDataConnectedNodes] = useState(false);
  const [dataVerified, setDataVerified] = useState(false);
  const [dataRegistered, setDataRegistered] = useState(false);

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) {
        // console.log('All data: ', response.data);
        setData(response.data);
        console.log(response.data);
        setDataTable(response.data.connected);
        setDataConnectedNodes(response.data.connectedNodes);
        setDataRegistered(response.data.scoreArray.length);
        setDataVerified(response.data.connected.length);
        console.log(response.data.connected);
      }
    };
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
                {dataVerified && <span>{dataVerified}</span>}
                Verified
              </li>
              <li>
                {dataRegistered && <span>{dataRegistered}</span>}
                Registered
              </li>
              <li>
                {dataConnectedNodes && <span>{dataConnectedNodes}</span>}
                Connected
              </li>
            </ul>
          </div>
        </div>
        <div className="box-main-area remove-all-padding">
          <div className="box-container-table">
            {dataTable && (
              <table id="date">
                <thead>
                  <tr>
                    {columns.map((e, index) => {
                      const { title, key } = e;
                      return <th scope="col" key={key}>{title}</th>;
                    })}
                  </tr>
                </thead>
                <tbody>
                  {/* <tr>.map</tr> */}
                  {dataTable.map((e, index) => {
                    const { address, id, score, tweetId, tweetUrl } = e;
                    return (
                      <tr key={id}>
                        <td data-type="score" data-label="score">{score}</td>
                        <td data-label="address">{address}</td>
                        <td data-label="id">{id}</td>
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
