#!/bin/bash
WEBVER=/home/pi/rtjam/version.txt
LOCALVER=/home/pi/rtjam/version.local.txt
cd /home/pi/rtjam
rm $WEBVER
wget -O $WEBVER localhost/pi/rust/version.txt
if [ "$?" -ne "0" ]; then
  echo "could not get version from server"
  exit 2
fi
./rtjam-sound --version > $LOCAL_VERSION
cmp -s $WEBVER $LOCALVER
if [ "$?" -ne "0" ]; then
  echo "Updating rtjam software"
  /usr/bin/systemctl stop rtjam-sound
  /usr/bin/mv rtjam-sound rtjam-sound.old
  /usr/bin/wget localhost/pi/rust/rtjam_sound
  /usr/bin/mv rtjam_sound rtjam-sound
  /usr/bin/chmod +x rtjam_sound
  /usr/bin/systemctl start rtjam-sound
else
  echo "No update needed"
fi
