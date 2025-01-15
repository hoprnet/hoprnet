	echo '{"packageName":"hoprd_sdk","projectName":"hoprd-sdk","packageVersion":"'$(shell ./scripts/get-current-version.sh docker)'","packageUrl":""}' >| /tmp/python-sdk-config.json

	mkdir -p ./hoprd-sdk-python/
	rm -rf ./hoprd-sdk-python/*

	swagger-codegen3 generate \
		-l python \
		-o hoprd-sdk-python \
		-i /tmp/openapi.spec.json \
		-c /tmp/python-sdk-config.json