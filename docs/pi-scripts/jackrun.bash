#!/bin/bash
NATION=http://rtjam-nation.com/pi/rust
WEBVER=version.txt
LOCALVER=version.local.txt
PROGRAM=rtjam_sound
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
# Check for soundin.cfg
if [ -f soundin.cfg ];
then
  INDEV=`cat soundin.cfg`
else
  INDEV=hw:CODEC
  echo $INDEV > soundin.cfg
fi
# make sure there was something in the file
if [ -z ${INDEV} ];
then 
  INDEV=USB 
fi
if [ -f soundout.cfg ];
then
  OUTDEV=`cat soundout.cfg`
else
  OUTDEV=$INDEV
fi
# make sure there was something in the file
if [ -z ${OUTDEV} ];
then 
  OUTDEV=$INDEV
fi
#
/usr/bin/aplay -l > devices.txt
JACK_NO_AUDIO_RESERVATION=1 /usr/bin/jackd -R -dalsa -r48000 -n 2 -p128 -C $INDEV -P $OUTDEV &
sleep 3
./$PROGRAM
