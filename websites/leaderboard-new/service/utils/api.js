import db, { FirebaseNetworkTables } from "./db";
import { HOPR_NETWORK } from "./env";

export async function getData(table) {
  try {
    const queryResponse = await db.getTable(HOPR_NETWORK, table);
    if (queryResponse.data) {
      return { data: queryResponse.data, status: 200 };
    } else {
      return { data: null, status: 500 };
    }
  } catch (e) {
    return { data: null, status: 500 };
  }
}

export async function getState() {
  return getData(FirebaseNetworkTables.state);
}

export async function getScore() {
  return getData(FirebaseNetworkTables.score);
}

export async function getAllData() {
  const [state, score] = await Promise.all([
    getData(FirebaseNetworkTables.state).then((res) => res.data),
    getData(FirebaseNetworkTables.score).then((res) => res.data),
  ]);

  const nodes = Object.entries(score).map(([id, score]) => {
    const node = state.connected.find((node) => node.id === id) || {};

    return {
      online: !!node.address,
      address: node.address || "?",
      id: id || "?",
      score: score || "?",
      tweetId: node.tweetId || "?",
      tweetUrl: node.tweetUrl || "?",
    };
  });

  state.nodes = nodes;

  return {
    data: state,
  };
}

export default { getState, getScore, getAllData };
