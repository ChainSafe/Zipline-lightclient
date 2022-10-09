
build: docker_image
	docker run -it --rm --name dev5 -v $(shell pwd):/code -v $(shell pwd)/../light-client-rs:/light-client-rs light-client-verification-mips/builder bash build.sh

docker_image:
	docker build . -t light-client-verification-mips/builder

