VERSION:=latest
SLUG:=ssd-benchmark

image-m1:
	docker build -t 5422m4n/${SLUG}:${VERSION} .
image:
	docker build --platform linux/amd64 \
				 -t 5422m4n/${SLUG}:${VERSION} .
publish:
	docker push 5422m4n/${SLUG}:latest
use:
	docker run --rm 5422m4n/${SLUG}