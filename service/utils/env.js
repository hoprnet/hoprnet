const HOPR_NETWORK = process.env["NEXT_PUBLIC_HOPR_NETWORK"] || "basodino";
const HOPR_DATABASE_URL =
  process.env["NEXT_PUBLIC_HOPR_DATABASE_URL"] || "hopr-coverbot";

module.exports = {
  HOPR_NETWORK,
  HOPR_DATABASE_URL,
};
