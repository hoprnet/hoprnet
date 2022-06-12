# Append here when new Rust WASM crate is added (the list is space separated)
PACKAGES = core-crypto
##########################################

PKG_DIRS = $(addsuffix /pkg, $(PACKAGES))
JS_FILES = $(foreach package,$(PACKAGES),$(package)/pkg/$(subst -,_,$(package)).js)

.PHONY := all clean test install

all: $(JS_FILES)
	@echo "All WASM modules built successfully"

test: $(JS_FILES)
	$(foreach pkg,$(PACKAGES),cargo test && wasm-pack test --node $(pkg))

$(JS_FILES):
	@echo "--- Building WASM module to '$(@D)' ... ---"
	mkdir -p $(@D)
	cd $(@D)/.. && wasm-pack build --target=nodejs
	@echo "--- Built WASM module in '$(@D)' ---"

install:
ifneq ($(PACKAGES),)
	@mkdir -p ../lib
	install $(foreach pkg, $(PKG_DIRS), $(pkg)/*.js) ../lib/
	install $(foreach pkg, $(PKG_DIRS), $(pkg)/*.ts) ../lib/
	install $(foreach pkg, $(PKG_DIRS), $(pkg)/*.wasm) ../lib/
endif

clean:
ifneq ($(PACKAGES),)
	rm -rf $(PKG_DIRS)
endif
