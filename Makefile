.POSIX:

PREFIX = /usr
XDG_CONFIG_HOME ?= "${HOME}/.config"

build:
	echo $(XDG_CONFIG_HOME)
	cargo build
	mkdir -p $(XDG_CONFIG_HOME)/rpm
	touch $(XDG_CONFIG_HOME)/rpm/storage.db
	touch $(XDG_CONFIG_HOME)/rpm/pass_hash
	chmod 777 $(XDG_CONFIG_HOME)/rpm/storage.db
	chmod 777 $(XDG_CONFIG_HOME)/rpm/pass_hash

testuj:
	mkdir -p "$(XDG_CONFIG_HOME)/rpm"

install:
	mkdir -p $(PREFIX)/bin
	cp target/debug/rpc $(PREFIX)/bin/rpc
	cp target/debug/rpd $(PREFIX)/bin/rpd
	cp rpm_rofi $(PREFIX)/bin/rpm_rofi
	chmod +x $(PREFIX)/bin/rpc
	chmod +x $(PREFIX)/bin/rpd
	chmod +x $(PREFIX)/bin/rpm_rofi

uninstall:
	rm $(PREFIX)/bin/rpc
	rm $(PREFIX)/bin/rpd
	rm $(PREFIX)/bin/rpm_rofi

.PHONY: install uninstall build
