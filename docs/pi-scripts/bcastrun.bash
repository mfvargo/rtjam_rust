#!/bin/bash
NATION=http://rtjam-nation.com/pi/rust-2
WEBVER=version.txt
LOCALVER=version.local.txt
PROGRAM=rtjam_broadcast
rm $WEBVER
wget -q -O $WEBVER $NATION/$WEBVER
if [ "$?" -ne "0" ]; then
  echo "could not get version from server"
else
  ./$PROGRAM --version > $LOCALVER
  cmp -s $WEBVER $LOCALVER
  if [ "$?" -ne "0" ]; then
    /usr/bin/wget -q -O $PROGRAM $NATION/$PROGRAM
    /usr/bin/chmod +x $PROGRAM
  fi
fi
./$PROGRAM