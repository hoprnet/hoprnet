#!/usr/bin/env bash
uv sync --no-dev
uv run --no-sync -m sdk.python.localcluster --config ./sdk/python/localcluster.params.yml --fully_connected --exposed
