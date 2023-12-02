IMG_PREFIX=ghcr.io/musergi
IMG_DEF_DIR=images

.PHONY: all
all:
	@LC_ALL=C $(MAKE) -pRrq -f $(firstword $(MAKEFILE_LIST)) : 2>/dev/null | awk -v RS= -F: '/(^|\n)# Files(\n|$$)/,/(^|\n)# Finished Make data base/ {if ($$1 !~ "^[#.]") {print $$1}}' | sort | grep -E -v -e '^[^[:alnum:]]' -e '^$@$$'

db-test: testing.db
	sqlx database setup

.PHONY: db-test
build:
	cargo build

.PHONY: image-osprei-git
image-osprei-git:
	docker build -t $(IMG_PREFIX)/osprei-git:latest - < $(IMG_DEF_DIR)/osprei-git.dockerfile

.PHONY: image-sqlx
image-sqlx:
	docker build -t $(IMG_PREFIX)/sqlx:latest - < $(IMG_DEF_DIR)/sqlx.dockerfile

.PHONY: image-sqlx-setup
image-sqlx-setup: image-sqlx
	docker build -t $(IMG_PREFIX)/sqlx-setup:latest - < $(IMG_DEF_DIR)/sqlx-setup.dockerfile

.PHONY: image-cargo-build
image-cargo-build:
	docker build -t $(IMG_PREFIX)/cargo-build:latest - < $(IMG_DEF_DIR)/cargo-build.dockerfile
