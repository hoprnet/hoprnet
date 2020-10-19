import debug from "debug";
import fetch from "isomorphic-fetch";
import { HOPR_DATABASE_URL } from "./env";

const log = debug("hopr-leaderboard:firebase");
const error = debug("hopr-leaderboard:firebase:error");

export const FirebaseNetworkSchema = {
  basodino: 'basodino',
};

export const FirebaseNetworkTables = {
  score: 'score',
  state: 'state'
};

class FirebaseDatabase {
  constructor() {
    this.databaseUrl = `https://${HOPR_DATABASE_URL}.firebaseio.com/`;
  }

  async resolveResponse(response) {
    if (response) {
      const json = await response
        .json()
        .catch((err) =>
          error(
            `- resolveResponse | json :: Error parsing data from response`,
            err
          )
        );
      log(`- resolveResponse | Retrieved json ${JSON.stringify(json)}`);
      return { data: json, status: 200 };
    } else {
      error(`- resolveResponse | Failed to retrieve data.`);
      return { data: null, status: 500 };
    }
  }

  async getSchema(schema) {
    try {
      log(`- getSchema | Retrieving schema ${schema} from ${this.databaseUrl}`);
      const response = await fetch(
        `${this.databaseUrl}${schema}.json`
      ).catch((err) =>
        error(`- getSchema | fetch :: Error retrieve data from database`, err)
      );
      return this.resolveResponse(response);
    } catch (err) {
      error(`- getSchema | catch :: Error retrieving data`, err);
      return { data: null, status: 500 };
    }
  }

  async getTable(schema, table) {
    try {
      log(
        `- getTable | Retrieving table ${table} within schema ${schema} from ${this.databaseUrl}`
      );
      const response = await fetch(
        `${this.databaseUrl}${schema}/${table}.json`
      );
      return this.resolveResponse(response);
    } catch (err) {
      error(`- getTable | catch :: Error retrieving data`, err);
      return { data: null, status: 500 };
    }
  }
}

export default new FirebaseDatabase();
