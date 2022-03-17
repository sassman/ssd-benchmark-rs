VERSION:=latest
SLUG:=ssd-benchmark-rs

login:
	echo ${CR_PAT} | docker login ghcr.io -u sassman --password-stdin
image-m1:
	docker build -t ghcr.io/sassman/${SLUG}:${VERSION} .
image:
	docker build --platform linux/amd64 \
				 -t ghcr.io/sassman/${SLUG}:${VERSION} .
publish:
	docker push ghcr.io/sassman/${SLUG}:${VERSION}
	# docker push 5422m4n/${SLUG}:latest
use:
	docker run --rm ghcr.io/sassman/${SLUG}