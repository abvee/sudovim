# Sudovim
A program that aims to make managing config files made at the root level
a lot simpler.

It tracks which config files you have changed and creates the path to that file
in `XDG_DATA_HOME/sudovim`, symlinking it to the original file.

This allows you to easily see which files you have changed from the default.

For example, if you change `/etc/default/grub`, it will create `etc/default`
folders in `XDG_DATA_HOME/sudovim` and a `etc/default/grub` symlink pointing to
the original file.
# Usage
Call the program the same way you would call `sudo vim`
```
$ sudovim <list of files>
```
This will launch your `$EDITOR` as root. Any files you change will then have
their path saved in `XDG_DATA_HOME/sudovim`
# Compiling and Installation
I've built with Rust 1.91.1 as of last testing with the same cargo version, so
use that if something goes wrong.
I know the Rust core lib is different, so idk which version that is
```
$ cargo build --release
```
PKGBUILD coming soon
# Unfinished things
* It doesn't track creating files from inside the editor. **Eg**: using
`:e` in vim. This would require ptrace shenanigans that I don't know how to use
* PKGBUILD
# Non Features
* You can add paths to `XDG_DATA_HOME/sudovim` yourself with `ln`. `sudovim`
probably will never manual adding of paths, because that just makes it a worse
frontend for `ln` and `mkdir`
