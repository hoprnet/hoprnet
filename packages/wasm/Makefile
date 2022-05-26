# Append here when new WASM crate is added
PACKAGES = common

BINARIES = $(foreach package,$(PACKAGES),$(package)/pkg/$(subst -,_,$(package)).js)

.PHONY := all clean test

all: $(BINARIES)
	@echo "WASM build completed"

test: $(BINARIES)
	$(foreach pkg,$(PACKAGES),wasm-pack test --node $(pkg))

$(BINARIES):
	@echo "--- Building WASM module to '$(@D)' ... ---"
	mkdir -p $(@D)
	cd $(@D)/.. && wasm-pack build --target=bundler
	@echo "--- Built WASM module in '$(@D)' ---"

clean:
	rm -rf $(addsuffix /pkg, $(PACKAGES))
