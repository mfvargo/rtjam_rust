#!/bin/bash
WEBVER=version.txt
LOCALVER=version.local.txt
cd /home/pi/rtjam
rm $WEBVER
wget -O $WEBVER localhost/pi/rust/version.txt
if [ "$?" -ne "0" ]; then
  echo "could not get version from server"
  exit 2
fi
echo "getting local version"
./rtjam-broadcast --version > version.local.txt
cmp -s $WEBVER $LOCALVER
if [ "$?" -ne "0" ]; then
  echo "Updating rtjam software"
  /usr/bin/systemctl stop rtjam-broadcast
  /usr/bin/mv rtjam-broadcast rtjam-broadcast.old
  /usr/bin/wget localhost/pi/rust/rtjam_broadcast
  /usr/bin/mv rtjam_broadcast rtjam-broadcast
  /usr/bin/chmod +x rtjam-broadcast
  /usr/bin/systemctl start rtjam-broadcast
else
  echo "No update needed"
fi