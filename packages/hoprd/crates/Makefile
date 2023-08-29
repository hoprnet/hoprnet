# Append here when new Rust WASM crate is added (the list is space separated)
PACKAGES = hoprd-hoprd hoprd-inbox hoprd-misc hoprd-keypair
##########################################

PKG_DIRS = $(addsuffix /pkg, $(PACKAGES))
JS_FILES = $(foreach package,$(PACKAGES),$(package)/pkg/$(subst -,_,$(package)).js)

.PHONY := all clean test install install-hoprd

all: $(JS_FILES)

test: $(JS_FILES)
	$(foreach pkg,$(PACKAGES),cargo test -p $(pkg) && WASM_BINDGEN_WEAKREF=1 WASM_BINDGEN_EXTERNREF=1 wasm-pack test --node `pwd`/$(pkg) && ) true

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

install-hoprd:
ifneq ($(PACKAGES),)
	@mkdir -p ../lib
	install hoprd-hoprd/pkg/*.js ../lib/
	install hoprd-hoprd/pkg/*.ts ../lib/
	install hoprd-hoprd/pkg/*.wasm ../lib/

	WASM_EXT_SUFFIX=$$(find ./hoprd-hoprd/pkg/snippets -type d | grep -oP '.+libp2p-wasm-ext-\K[a-f0-9]+$$' | uniq); \
	mkdir -p ../lib/snippets/libp2p-wasm-ext-$${WASM_EXT_SUFFIX}/src/; \
	install ./hoprd-hoprd/pkg/snippets/libp2p-wasm-ext-$${WASM_EXT_SUFFIX}/src/*.js ../lib/snippets/libp2p-wasm-ext-$${WASM_EXT_SUFFIX}/src/; \
	echo "{}" >| ../lib/snippets/libp2p-wasm-ext-$${WASM_EXT_SUFFIX}/src/package.json
endif

clean:
ifneq ($(PACKAGES),)
	rm -rf $(PKG_DIRS)
endif
