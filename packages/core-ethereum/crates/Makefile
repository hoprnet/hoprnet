# Append here when new Rust WASM crate is added (the list is space separated)
PACKAGES = core-ethereum-db core-ethereum-misc core-ethereum-indexer
##########################################

PKG_DIRS = $(addsuffix /pkg, $(PACKAGES))
JS_FILES = $(foreach package,$(PACKAGES),$(package)/pkg/$(subst -,_,$(package)).js)

.PHONY := all clean test install

all: $(JS_FILES)

test: $(JS_FILES)
	$(foreach pkg,$(PACKAGES),cargo test --no-default-features -p $(pkg) && WASM_BINDGEN_WEAKREF=1 WASM_BINDGEN_EXTERNREF=1 wasm-pack test --node `pwd`/$(pkg) && ) true

$(JS_FILES):
	mkdir -p $(@D)
	WASM_BINDGEN_WEAKREF=1 WASM_BINDGEN_EXTERNREF=1 wasm-pack build --target=bundler `pwd`/$(@D)/..

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
