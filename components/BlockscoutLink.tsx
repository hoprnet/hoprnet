function BlockscoutLink({ id, children }) {
  return (
    <a
      target="_blank"
      href={"https://blockscout.com/poa/xdai/address/" + id + "/transactions"}
    >
      {children}
    </a>
  );
}

export default BlockscoutLink;
