# Setup a pi with 64 bit os

install the latest 64 bit pi os https://www.raspberrypi.com/software/

make sure pi os is up to date

```
sudo apt update
sudo apt upgrade
```

## To run the software:

```
sudo apt install jackd2
sudo apt install git
sudo apt install libssl1.1
git clone https://github.com/mfvargo/rtjam_rust.git
cd rtjam_rust
```

To install the sound component you run

```
make install-sound
```

To install the broadcast component you run

```
make install-broadcast
```

The [`Makefile`](/Makefile) is super simple. It installs the service files
for systemctl and retrieves the release built executables from rtjam-nation.com

Note that almost all of this stuff is to set it up so the software runs as a service.

It creates a directory called rtjam under /home/pi and then copies files there.\

## To build the software

- install rust - curl https://sh.rustup.rs -sSf | sh
- install libssl - sudo apt install libssl-dev
- install libjack-dev - sudo apt install libjack-dev
- git clone https://github.com/mfvargo/rtjam_rust.git
- cd rtjam_rust
- cargo build etc ( or you can make )

## Notes about audio run choices

The library comes with two ways to pump audio to/from the sound device.  One is using jack and the other is using ALSA library directly

Currently the direct I/O via alsa works using the hw:DEVICE_ID monikor only on devices that support 16 bit signed samples.  THe jack software does all the required conversions to make everything into f32 buffers.

The alsa thread works great with the Sigma Codec on the i2s bus and with the usb audio devices from Behringer.