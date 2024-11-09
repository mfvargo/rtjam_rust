#!/bin/bash
NATION=http://rtjam-nation.com/pi/rust
WEBVER=version.txt
LOCALVER=version.local.txt
PROGRAM=rtjam_sound
# This is how we will fetch from the network
WGET='wget -q --tries=2 --timeout=3'

# if RUST_LOG is not set, set it to info, otherwise use the prior setting
if [ -z ${RUST_LOG} ]; then
  export RUST_LOG=info
fi
echo "RUST_LOG is set to ${RUST_LOG}"

if [ -z $SKIP_UPDATE ]; then
  # Step one is to get the version of the software on the web
  rm $WEBVER
  $WGET -O $WEBVER $NATION/$WEBVER
  if [ "$?" -ne "0" ]; then
    # not able to get the web version, so we will just continue on and run
    # This is likely the no network scenario or the server is down for maintenance
    echo "Cannot contact the nation:  no version from $NATION"
  else
    # Get the version number out of the local program
    ./$PROGRAM --version > $LOCALVER
    if [ "$?" -ne "0" ]; then
      # This is an error so the rtjam_sound is corrupt.  Do the recovery here....
      echo "program will not give version.  something wrong with it"
      # TODO:  Should we do this here?  Get the version stashed in rollback
      # cp -f rollback/$PROGRAM $PROGRAM
    fi
    # compare the local version with the one on the web
    cmp -s $WEBVER $LOCALVER
    if [ "$?" -ne "0" ]; then
      # There is a new version on the web  Stash the old one
      echo "New version available, gonna download"
      cp -f $PROGRAM $PROGRAM.rollback
      $WGET -O $PROGRAM $NATION/$PROGRAM
      if [ "$?" -ne "0" ]; then
        # Something went wrong fetching the program file.  rollback
        echo "failed to download new version.  Rollback"
        cp -f $PROGRAM.rollback $PROGRAM
      else
        # We successfully downloaded the program
        chmod +x $PROGRAM
        # Make sure the executable will run a version
        ./$PROGRAM --version 
        if [ "$?" -ne "0" ]; then
          # Yikes, the thing we downloaded wont run here.  better rollback
          echo "Something wrong with new version, rollback!"
          cp -f $PROGRAM.rollback $PROGRAM
        fi
      fi
    else
      echo "Software is up to date"
    fi
  fi
else
  echo "Skipping Update"
fi

# Check for soundin.cfg
if [ -f soundin.cfg ];
then
  INDEV=`cat soundin.cfg`
else
  INDEV=plughw:CODEC
  echo $INDEV > soundin.cfg
fi
# make sure there was something in the file
if [ -z ${INDEV} ];
then 
  INDEV=plughw:CODEC
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
