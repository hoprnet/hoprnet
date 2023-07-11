.POSIX:

all: help

.PHONY: start-structurizr
start-structurizr: ## start structurizr Docker container
	@echo
	@echo "Visit http://localhost:8080/workspace/documentation once the process has started"
	@echo
	docker run --pull always -it --rm -p 8080:8080 \
		-v `pwd`:/usr/local/structurizr structurizr/lite

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
