```eval_rst
.. ATTENTION::
   This page is under construction and missing many of the available HOPRd flags.
```

## Log forwarding

When passed the flag `--forwardLogs`, all actionable commands (e.g. commands you run directly in your node) will be forwarded to a logging [sink](<https://en.wikipedia.org/wiki/Sink_(computing)>). The following providers are currently supported as sinks for your commands.

### Providers

- [Ceramic Network (default)](https://ceramic.network/). Ceramic is a decentralized, open source platform for creating, hosting, and sharing streams of data. Currently, we are using Ceramic within HOPR nodes as a way to share your commands for debugging and logging purposes, as an easy way to provide support for your nodes without having to rely on a centralised party. See all your logs from the URL given at the beginning of your Admin panel startup. Using Ceramic, your logs can be browsable via [Documint](https://documint.net/) or [Tiles](https://tiles.ceramic.community/).
