# Rusty Password Manager (RPM)

Simple client/server password manager that keeps you logged in for few minutes and lets you get your login or password into clipboard.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes. See deployment for notes on how to deploy the project on a live system.

### Prerequisites

All necessary packages are handled by rust's cargo. Only thing you need, is to have rofi installed if you want to use it. Program functions with CLI as well, rofi is just the most handy way to use it.

### Installing

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

### Usage
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

### TODO
* avoid user to simply remove password_hash and use init
* allow changing of records
* change validation messages for user
* add another features to rofi

### License

This project is licensed under the DO_WTF_U_WANT_WITH_IT License.

### Acknowledgments

Project is under initial development. PRs are welcome !