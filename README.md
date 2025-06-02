# Qop - Incremental file opening menu
This project was literally made in a day and probably isn't ready for you.
It's allows you to build and execute a command incrementally. I use it as a rofi alternative since I find myself opening like 5 programs from the entire list.

## Usage:

Qop searches for TOML configuration files in '~/.config/qop/{mode}.toml'

Qop can be called in a specific 'mode' by running:

``` sh
qop -s mode

or 

qop --show mode
```

This will display a menu allowing you to run the shortcuts defined in your 'mode' file

``` text
~~  Home > Home                 emacs      ~/         SOME_ENV
~de Home > Dotfiles > Emacs     emacs      ~/.config/emacs
~dt Home > Dotfiles > WithArgs  emacs -nw  ~/.config
~dd Home > Dotfiles > DiffPath  emacs       /home/some/file
ff  Fun                         ls -a       /
```

## Config file

This would be a verbose config the for the menu above
``` toml
[home]
key = "~"

[home.home]
key = "~"
exec = "emacs"
path = "~/"
env = ["SOME_ENV=123"]

[home.dotfiles]
key = "~"

[home.dotfiles.emacs]
key = "e"
exec = "emacs"
path = "~/.config/emacs/"

[home.dotfiles.withargs]
key = "t"
exec = "emacs"
args = ["-nw"]
path = "~/.config"

[home.dotfiles.diffpath]
key = "d"
exec = "emacs"
path = "/home/some/file"

[fun]
key = "ff"
exec = "ls"
args = ["-a"]
path = "/"
```

Commands are composable so you could simplify the config by moving common parts up the tree
``` toml
[home]
key = "~"
exec = "emacs"
path = "~/"
env = ["SOME_ENV=123"]

[home.home]
key = "~"

[home.dotfiles]
key = "~"
# You can append to a path with a '+' prefix
# Any '+' or '/' character will be trimmed from this value so you don't need to worry about slashes
path = "+.config"

[home.dotfiles.emacs]
key = "e"
# You can also end a path with '/' or not
path = "+.emacs.d/"

[home.dotfiles.withargs]
key = "t"
args = ["-nw"]

[home.dotfiles.diffpath]
key = "d"
exec = "emacs"
# You can simply override the path if you want
# You can't do the same thing with arguments or environment so there's could be some limitations
path = "/home/some/file"

[fun]
key = "ff"
exec = "ls"
args = ["-a"]
path = "/"
```

There's a reserved 'global' section where you can specify any property other than 'key' and you can customize colors.
``` toml
[global]
background = "#121212"
foreground = "#cec0af"
highlight_accent = "#bf6601"
highlight = "#191511"
accent = "#663702"
faded = "#222222"
```

You could use this as a app launcher with a bit of tinkering.
The config file is simple but you need a way to run a terminal as a popup or floating window.

``` toml
# ~/.config/qop/run.toml
[firefox]
key = "fi"
exec = "librewolf"

[mail]
key = "ma"
exec = "thunderwolf"

[gimp]
key = "gi"
exec = "gimp"
```

Hyprland example:
```
# hyprland.conf

bind=SUPER, Q, exec, alacritty --title=qop -e qop -s run
windowrule = float,        title:qop,
windowrule = size 600 400, title:qop,
```
This runs an instance of alacritty as a popup and calls qop on it

## Building
``` sh
cargo build --release

# With Cargo in $PATH
cargo install --path .

# OR

# Fuck it, copy the binary
cp target/release/qop /usr/local/bin

# OR

# Something else, I believe in you!
```
