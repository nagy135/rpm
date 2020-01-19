#!/bin/bash

rpc=target/debug/rpc
rpd=target/debug/rpd

$rpc get test > /dev/null
if [[ $? -gt 0 ]]; then
    pass=$(echo "$keys" | rofi -dmenu -password -theme input.rasi -i -p "Master Password")
    $rpc validate $pass
    [[ $? -gt 0 ]] && notify-send -i none -t 2000 "RPM" "wrong password" && exit 1
fi


keys=$($rpc list)


chosen=$(echo "$keys" | rofi -dmenu -theme "~/.config/rofi/tmux.rasi" -i -p "Choose password key")

if [[ $chosen != "" ]]; then
    echo $($rpc get $chosen) |  tr -d '\n' | xclip -sel clipboard
fi
