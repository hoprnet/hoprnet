# Append here when new Rust WASM crate is added (the list is space separated)
PACKAGES = core-crypto core-hopr core-misc core-mixer core-network core-path core-packet core-strategy core-types
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

	WASM_EXT_SUFFIX=$$(find ../lib/ -type f -exec grep -oP 'libp2p-wasm-ext-\K\w+' {} \; | uniq); \
	mkdir -p ../lib/snippets/libp2p-wasm-ext-$${WASM_EXT_SUFFIX}/src/; \
	install ../../../vendor/cargo/libp2p-wasm-ext*/src/*.js ../lib/snippets/libp2p-wasm-ext-$${WASM_EXT_SUFFIX}/src/; \
	echo "{}" >| ../lib/snippets/libp2p-wasm-ext-$${WASM_EXT_SUFFIX}/src/package.json
endif

clean:
ifneq ($(PACKAGES),)
	rm -rf $(PKG_DIRS)
endif
