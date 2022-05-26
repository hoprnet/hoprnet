# Append here when new WASM crate is added
PACKAGES = hopr-wasm-common

BINARIES = $(foreach package,$(PACKAGES),$(package)/pkg/$(subst -,_,$(package)).js)

.PHONY := all clean test

all: $(BINARIES)
	@echo "WASM build completed"

test: $(BINARIES)
	$(foreach pkg,$(PACKAGES),wasm-pack test --node $(pkg))

$(BINARIES):
	@echo "--- Building WASM module '$(@D)' ... ---"
	mkdir -p $(@D)
	cd $(@D)/.. && wasm-pack build --target=bundler
	@echo "--- Built WASM module '$(@D)' ---"

clean:
	rm -rf $(addsuffix /pkg, $(PACKAGES))
