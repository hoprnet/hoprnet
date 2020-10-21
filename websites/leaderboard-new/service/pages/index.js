import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import BoxRemember from "../components/micro-components/box-remember";

import api from "../utils/api";

const columnsDefaults = [
  {
    title: "online",
    dataIndex: "online",
    key: "online",
    className: 'sortBy asc',
  },
  {
    title: "address",
    dataIndex: "address",
    key: "address",
    className: 'sortBy',
  },
  {
    title: "id",
    dataIndex: "id",
    key: "id",
    className: 'sortBy ',
  },
  {
    title: "score",
    dataIndex: "score",
    key: "score",
    className: 'sortBy',
  },
  {
    title: "tweetUrl",
    dataIndex: "tweetUrl",
    key: "tweetUrl",
  },
];

export default function Home() {
  const [data, setData] = useState(undefined);
  const [columns, setColumns] = useState(columnsDefaults);

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

  const getIntBase = key => {
    switch(key) {
      case 'address':
        return 16;
      case 'id':
        return 36;
      default:
        return 10;
    }
  };

  const onClickSort = key => {
    let sSort = '',
      aColumns = [...columns];

    aColumns.map(item => {
      if (item.key === key) {
        sSort = item.className.replace('sortBy', '').trim();
        sSort = sSort === 'asc' ? 'desc' : 'asc';
      }
      if (item.className !== undefined) {
        item.className = 'sortBy'
      }
    });
    aColumns.find(item => item.key === key).className = 'sortBy ' + sSort;

    let aNew = { ...data };
    aNew.nodes = aNew.nodes.sort((a, b) => {
      let iBase = getIntBase(key),
        convertA = parseInt(a[key], iBase),
        convertB = parseInt(b[key], iBase)

      if (sSort === 'asc') {
        return convertB - convertA;
      } else {
        return convertA - convertB;
      }
    });

    setData(aNew);
    setColumns(aColumns);
  };

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
                <colgroup span="4" class="columns"></colgroup>
                <thead>
                  <tr>
                    {columns.map(e => {
                      const { title, key, className } = e;
                      return (
                        <th
                          className={className}
                          onClick={className ? () => onClickSort(key) : ''}
                          scope="col"
                          key={key}
                        >
                          {title}
                        </th>
                      );
                    })}
                  </tr>
                </thead>
                <tbody>
                  {nodes.map((e) => {
                    const { online, address, id, score, tweetUrl } = e;
                    return (
                      <tr key={id}>
                        <td className="icon-help-online" data-label="online"><div className={[online ? "online" : "offline"]}></div></td>
                        <td data-label="address">
                          <a  
                          className="table-link-on"
                          target="_blank"
                          href={'https://explorer.matic.network/address/'+address}
                          rel="noopener noreferrer">{address}
                          </a>  
                        </td>
                        <td data-label="id">
                        {id}
                        </td>
                        <td data-type="score" data-label="score">
                          {score}
                        </td>
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
           <BoxRemember/>
        </div>
      </div>
    </Layout>
  );
}
