# H(ost)I(nfo) - rust version

cli dedicated to display infos about your host at the start of a shell session.
it works on linux (osx need to be tested) and requires a terminal with unicode and 256+ colors support.

## --help
```
hi 0.1.0

syntax: hi [-options] [text]

default text is your hostname

options:
-fg=color         foreground color/gradient
-bg=color         background color
-c[olor]=1|16|256 set color mode
-1|16|256         set color mode as well
-m[ono]           set color mode to 1
-font=default     font name (available: default,hashtag)
-s[mall]          set small mode (default font only)
-i[nfo]=kumic     choose info to display
-ssh=remote       ssh hostname

infos can be given as follow:
k=kernel   u=uptime   c=cpumodel
m=memory   i=ip

color can be given as follow:
name          eg: green
d,d,d         eg: 0,255,0
h:h:h         eg: 0:ff:0
hhhhhh        eg: 00ff00
hhh           eg: 0f0

gradient can be given as follow:
color-color   eg: white-blue
                  fff-00f

available colors: black,bright_black,red,bright_red,green,bright_green,yellow,bright_yellow,blue,bright_blue,magenta,bright_magenta,cyan,bright_cyan,white,bright_white,grey
available fonts: default,hashtag
```

## build

```
cargo build --release
```