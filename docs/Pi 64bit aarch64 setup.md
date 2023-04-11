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
- install libjack-dev - sudo apt install jackd2 libjack-dev
- git clone https://github.com/mfvargo/rtjam_rust.git
- cd rtjam_rust
- cargo build etc ( or you can make )
