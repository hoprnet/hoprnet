import { getState } from "../../utils/api";

export default async (_req, res) => {
  res.statusCode = 200;
  const response = await getState();

  if (response.data) {
    const data = response.data;
    return res.json(data);
  } else {
    return res.json({});
  }
};
