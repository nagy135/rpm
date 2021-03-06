# Rusty Password Manager (RPM)

Simple client/server password manager that keeps you logged in for few minutes and lets you get your login or password into clipboard.

## State

Just recently started working on this again, to substitute file database to sqlite to make this much less about parsing files and more password management.

## Getting Started

This project exists because at work I often connect to different machines and gui interfaces and I wanted simple rofi-based solution that would get me credentials into clipboard to simply paste it there. In the same time I wanted to get into rust language, so i made this. Whole idea is to get the password safely and without typing master password constantly. Therefore rpm is separated into two binaries, one that runs in the background (rpd - daemon - server) that holds your password and does all the heavy lifting and client (rpc) that simply communicates with it. Prefered way of using rpm is with given rofi bash script.

## Prerequisites

All necessary packages are handled by rust's cargo. Only thing you need, is to have rofi installed if you want to use it. Program functions with CLI as well, rofi is just the most handy way to use it.

## Safety

Rpm uses symmetric encryption via AES (Advanced Encryption Standard). This means that there exists one key (user's master password) that is used both to encrypt the data, as well as to decrypt it. Rpd (daemon part of the program) holds this key internally and invalidates it after timeout ran in separate thread.

## Installing

Installing process is very simple.

```
git clone "https://github.com/nagy135/rpm"
```

and then

```
cd rpm
make
sudo make install
```

Make sure your `XDG_CONFIG_HOME` environment variable is set. This is the location where all the data is stored, as well as themes. Unless you changed it, it should point to `~/.config`.

## Usage
Program consists of 2 parts:
* rpc - client
* rpd - daemon

You first need to run daemon in the background:
```
rpd &
```

Then you should set your initial password with:
```
rpc init
```

After your password is set, you need to validate:
```
rpc validate
```

This will prompt you for your password and validate your actions for next 10 minutes.
After that you can create new records:
```
rpc new my_key my_password
rpc new my_key my_login my_password
```
**Your shell might interpret some symbols differently**...enclose arguments with special characters in single quotes. This should solve all the cases except for cases where you want single quote inside ur password/login. Then you need to escape it with backslash:
```
# wont work (just some of problematic symbols)
rpc new my_key special_*$#!:_symbols
# will work
rpc new my_key 'special_*$#!:_symbols'
```

where second argument can be password, or login...you can either use it just to store password, or login as well.
Data is retrieved by:
```
rpc get my_key
rpc get my_key -l
```
This will print your password (or login, if record has one and flag -l is specified) to stdout.

You also might need to list all the keys to choose from:
```
rpc list
```

## Rofi
If rofi is installed, you can use `rpm_rofi` script to do all of above via rofi interface.
If validation is needed, rofi will give you prompt. If not you can choose from keys and retrieve password (or login with `rpm_rofi -l`) to the clipboard.
Rofi script supports all of the functionality, try `rpm_rofi --help`

## Contributing
I m planning to improve this password manager for my personal use, but any PRs are welcome and encouraged. This project gets you into multiple different topics you might want to know about rust:
* Encryption
* Multithreading (with shared memory)
* Server/Client architecture
* ...

This project was created with multiple weeks-long breaks, therefore my ideas might be inconsistent across entire project. I plan on refactoring it in future, but keep it in mind !

## TODO
* ~~avoid user to simply remove password_hash and use init (probably just wipe records if init is used)~~
* ~~allow changing of records~~
* ~~allow deletion of records~~
* ~~add another features to rofi~~
* ~~provide .rasi rofi themes in user's config for modification~~
* change validation messages for user
* modifiable timeout delay

# Restart
After year break I m going to update this project. Db will be changed from file to sqlite db to make datastorage little bit less awkward. And a lot of refactoring.

## TODO
* test functionality
* store data in sqlite db
* refactor

## License

This project is licensed under the DO_WTF_U_WANT_WITH_IT License.

## Acknowledgments

Project is under initial development. PRs are welcome !
