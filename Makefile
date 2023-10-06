.PHONY: test
test:
	cargo test

.PHONY: image
image:
	docker build -t osprei:latest .

.PHONY: container
container:
	docker run \
		-p 8081:8080 \
		-d \
		--rm \
		--name osprei \
		-e RUST_LOG=osprei=info,api=info \
		-v /opt/osprei:/opt/osprei \
		-v /run/docker.sock:/run/docker.sock \
		osprei:latest

.PHONY: dataset
dataset:
	cargo run \
		--bin osprei-cli \
		-- \
		--server http://localhost:8081 \
		add \
		--name osprei-test \
		--path ci/test.json
