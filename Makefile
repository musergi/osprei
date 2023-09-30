.PHONY: test
test:
	cargo test

.PHONY: image
image:
	docker build -t osprei:latest .

.PHONY: container
container:
	docker run \
		-p 8080:8080 \
		-d \
		--rm \
		--name osprei \
		-v /opt/osprei:/opt/osprei \
		-v /run/docker.sock:/run/docker.sock \
		osprei:latest
