import React, { useState, useEffect } from "react";
import Layout from "../components/layout/layout.js";
import api from "../utils/api";

const columnsDefaults = [
  {
    title: "score",
    dataIndex: "score",
    key: "score",
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
    className: 'sortBy',
  },
  {
    title: "tweetUrl",
    dataIndex: "tweetUrl",
    key: "tweetUrl",
    className: '',
  },
];

export default function HoprAllocation() {
  const [data, setData] = useState(undefined);
  const [columns, setColumns] = useState(columnsDefaults);

  useEffect(() => {
    const fetchData = async () => {
      const response = await api.getAllData();
      if (response.data) {
        const allNodes = response.data.nodes.sort((a, b) => b.score - a.score),
          nodes = allNodes.slice(0, allNodes.length > 6 ? 5 : allNodes.length);

        setData(nodes);
      };
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
      item.className = 'sortBy'
    });
    aColumns.find(item => item.key === key).className = 'sortBy ' + sSort;

    let aNew = [...data];
    aNew = aNew.sort((a, b) => {
      let iBase = getIntBase(key),
        convertA = parseInt(a[key], iBase),
        convertB = parseInt(b[key], iBase)

      if (sSort === 'asc') {
        return convertA - convertB;
      } else {
        return convertB - convertA;
      }
    });
    console.log('Sort nodes:', aNew);

    setData(aNew);
    setColumns(aColumns);
  };

  return (
    <Layout>
      <div className="box special-table-top">
        <div className="box-top-area">
          <div>
            <div className="box-title">
              <h1>Hopr Allocation</h1>
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
                  {data.map((item) => {
                    const { id, address, score,  tweetUrl } = item;
                    return (
                      <tr key={id}>
                         <td data-type="score" data-label="score">
                         <span>
                            <img
                              src="/assets/icons/top.svg"
                              alt="hopr Top ASSETS"
                            />
                          </span>
                          {score}
                        </td>
                        <td data-label="address">{address}</td>
                        <td data-label="id">{id}</td>
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
