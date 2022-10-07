
build: docker_image
	docker run -it --rm --name dev -v $(shell pwd):/code rust-in-my-cannon/builder bash build.sh

docker_image:
	docker build . -t rust-in-my-cannon/builder

