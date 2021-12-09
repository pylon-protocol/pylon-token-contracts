PWD := $(shell pwd)
BASENAME := $(shell basename $(PWD))

all: build deploy

build:
	docker run --rm -v "$(PWD)":/code \
  	  --mount type=volume,source="$(BASENAME)_cache",target=/code/target \
  	  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  	  cosmwasm/workspace-optimizer:0.12.4

deploy:
ifndef network
	yarn --cwd ./deployer start -d ../artifacts
else
	yarn --cwd ./deployer start -d ../artifacts --network $(network)
endif

deploy-columbus:
	make deploy network=columbus

deploy-bombay:
	make deploy network=bombay

deploy-local:
	make deploy network=local

publish:
	cargo publish --manifest-path ./packages/pylon_token/Cargo.toml
