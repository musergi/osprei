.PHONY: test
test:
	cargo test

.PHONY: image
image:
	docker build -t osprei:latest .

.PHONY: image-push
image-push:
	docker tag osprei:latest ghcr.io/musergi/osprei:latest
	docker push ghcr.io/musergi/osprei:latest
	docker rmi ghcr.io/musergi/osprei:latest

.PHONY: image-push-multiplatform
image-push-multiplatform:
	docker buildx build --platform linux/amd64,linux/arm/v7 -t ghcr.io/musergi/osprei:latest --push .

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
