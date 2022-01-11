import { ENDPOINTS, BASE_URL } from "./consts";
import { createClient } from "../client";

const client = createClient(BASE_URL);

export const getReleases = () => client.get(ENDPOINTS.releases);
