# Topskud

Pronunciation /ˈtɒpˌskʊd/
Danish: [ˈtʰʌb̥ˌsg̊uð]

A top-down shooter game.

## Build requirements

The below are adaptations of the documentation on `ggez`. Go there for more (or less) information.

## Windows

Should just be able to compile. MSVC toolchain works best.

## Linux

### Debian

The following packages are required:

```sh
apt install libasound2-dev libudev-dev pkg-config
```

### Redhat

Same libraries as Debian, slightly different names. On CentOS 7, at
least you can install them with:

```sh
yum install alsa-lib-devel
```
