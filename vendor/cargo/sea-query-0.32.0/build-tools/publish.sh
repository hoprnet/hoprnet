#!/bin/bash
set -e

# # publish `sea-query-attr`
# cd sea-query-attr
# cargo publish
# cd ..

# # publish `sea-query-derive`
# cd sea-query-derive
# cargo publish
# cd ..

# publish `sea-query`
cargo publish --allow-dirty

# publish `sea-query-binder`
cd sea-query-binder
cargo publish --allow-dirty
cd ..

# publish `sea-query-rusqlite`
cd sea-query-rusqlite
cargo publish --allow-dirty
cd ..

# publish `sea-query-postgres`
cd sea-query-postgres
cargo publish --allow-dirty
cd ..
