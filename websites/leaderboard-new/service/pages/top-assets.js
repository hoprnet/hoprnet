import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import api from "../utils/api";

export default function TopAssets() {
  const [data, setData] = useState([]);

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) {
        if (response.data.connected) {
          let connected = response.data.connected.sort((a, b) => b.score - a.score);
          if (connected.length) {
            setData(connected.slice(0, connected.length > 6 ? 5 : connected.length));
          }
        }
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
      <div className="box special-table-top">
        <div className="box-top-area">
          <div>
            <div className="box-title">
              <h1>Top Assets</h1>
            </div>
          </div>
        </div>
        <div className="box-main-area remove-all-padding aux-add-top ">
          <div className="box-container-table">
              {data && (
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
                  {data.map(item => {
                    const { id, address, score, tweetId, tweetUrl } = item;
                    return (
                      <tr key={id}>
                        <td data-type="score" data-label="score">
                          <span > <img src="/assets/icons/top.svg" alt="hopr Top ASSETS" /></span>
                          {score}</td>
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
