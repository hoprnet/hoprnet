

run:
	k6 run dist/constant-traffic.js


hoprd:
	docker run --pull always --restart on-failure -m 2g \
	-ti -v $$HOME/.hoprd-db-bogota:/app/hoprd-db \
	-p 9091:9091 -p 3000:3000 -p 3001:3001 -e DEBUG="hopr*" \
	gcr.io/hoprassociation/hoprd:bogota \
	--environment monte_rosa \
	--init --api --admin \
	--identity /app/hoprd-db/.hopr-id-bogota \
	--data /app/hoprd-db \
	--password 'open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0' \
	--apiHost "0.0.0.0" --apiToken '^MYtoken4testing^' \
	--adminHost "0.0.0.0" \
	--healthCheck --healthCheckHost "0.0.0.0"

start: 
	./scripts/setup-local-cluster.sh -i topologies/full_interconnected_cluster.sh