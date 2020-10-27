const params = new URLSearchParams({
  text:
    "Signing up to earn $HOPR on the #Basodino testnet. My @hoprnet address is: ",
  // hashtags: ["HOPRnetwork", "Basodino"].join(","),
  related: ["hoprnet"].join(","),
});

const TweetBasodino = ({ children }) => (
  <a
    href={`https://twitter.com/intent/tweet?${params}`}
    target="_blank"
    rel="noreferrer"
  >
    {children}
  </a>
);

export default TweetBasodino;
