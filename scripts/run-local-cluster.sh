python -m venv .venv
source .venv/bin/activate
pip install -r tests/requirements.txt

python -m sdk.python.localcluster --config ./sdk/python/localcluster.params.yml --fully_connected --exposed
