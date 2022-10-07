# Rust in my Cannon 🦀💣💥

This repo contains a build system and a minimal Rust program for building MIPS binaries that are executable in the context of [Optimism Cannon](https://github.com/ethereum-optimism/cannon). 

## Usage

Build a binary `.bin` output for this rust crate by running

```shell
make
```

Alternatively if you want to experiment in the build environment you can with
	
```shell
make docker_image
docker run -it --rm --name dev -v $(pwd):/code rust-in-my-cannon/builder bash
```

and from there you can run 

```shell
./build.sh
```
to produce the output

## Credits

All of the hard work was done by @pepyakin in their [Cannon fork](https://github.com/pepyakin/rusty-cannon/). This just pulls out the relevant pieces and adds a few quality of life improvements to the build system.
