#!/bin/bash
NATION=http://rtjam-nation.com/pi/rust
WEBVER=/home/pi/rtjam/version.txt
LOCALVER=/home/pi/rtjam/version.local.txt
cd /home/pi/rtjam
rm $WEBVER
wget -O $WEBVER $NATION/version.txt
if [ "$?" -ne "0" ]; then
  echo "could not get version from server"
  exit 2
fi
echo "getting local version"
./rtjam_broadcast --version > version.local.txt
cmp -s $WEBVER $LOCALVER
if [ "$?" -ne "0" ]; then
  echo "Updating rtjam software"
  /usr/bin/systemctl stop rtjam-broadcast
  /usr/bin/wget -O rtjam_sound $NATION/rtjam_broadcast
  /usr/bin/chmod +x rtjam_broadcast
  /usr/bin/systemctl restart rtjam-broadcast
else
  echo "No update needed"
fi