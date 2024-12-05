# Sudovim
A program that aims to make managing config files made at the root level
a lot simpler.

It tracks which config files you have changed and creates the path to that file
in `XDG_DATA_HOME/sudovim`, symlinking it to the original file.

This allows you to see which files you have changed from the default.

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
You need minimally have zig-0.12.0 installed to compile the program. Build it
with:
```
$ zig build
```
Install it to a specific directory with:
```
# zig build -p /usr/local
```
This will install it to `/usr/local/bin`
# Unfinished things
* If you create a file from the editor, it doesn't factor that in. **Eg**: using
`:e` in vim
* If you give it files that don't exist yet. For example calling `vim somefile`
will create `somefile` when you write to it. sudovim crashes when you call
`sudovim <file that doesn't exist>`
# Non Features
* You can add paths to `XDG_DATA_HOME/sudovim` yourself with `ln`. `sudovim`
probably will never manual adding of paths, because that just makes it a worse
frontend for `ln` and `mkdir`
