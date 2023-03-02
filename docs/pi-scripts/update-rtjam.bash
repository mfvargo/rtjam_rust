#!/bin/bash
NATION=http://rtjam-nation.basscleftech.com/pi/rust
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
./rtjam_sound --version > version.local.txt
cmp -s $WEBVER $LOCALVER
if [ "$?" -ne "0" ]; then
  echo "Updating rtjam software"
  /usr/bin/systemctl stop rtjam_sound
  /usr/bin/wget $(NATION)/rtjam_sound
  /usr/bin/chmod +x rtjam_sound
  /usr/bin/systemctl start rtjam_sound
else
  echo "No update needed"
fi
