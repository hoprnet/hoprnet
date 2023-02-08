# Load Testing

This package is responsible of creating a set of load testing scenarios to be used to check performance and capacity of Hoprd network

## Use cases

- Generate high peaks of load to test network stability at those circumstances
- Generate high peaks of load to test hoprd node stability at those circumstances
- Generate cover traffic on nodes while there is a better cover traffic approach implemented within the hoprd node
- Generate constant traffic on nodes to test the network and their nodes against long running periods

https://github.com/grafana/xk6-output-prometheus-remote
https://k6.io/docs/results-output/real-time/prometheus-remote-write/
https://k6.io/docs/get-started/results-output/

https://github.com/hoprnet/hopr-network-dashboard/blob/main/be-scripts/modules/hopr-sdk.js



# Setup development environment

Here are the most useful commands:
- `yarn install`: Install dependencies
- `npm run build`: Build source code
- `npm run cluster:start`: Start local cluster
- `npm run test`: Execute tests locally
- `npm run cluster:stop`: Stops the local cluster