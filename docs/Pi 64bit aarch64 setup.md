Setup a pi with 64 bit os

make sure pi os is up to date

- sudo apt update
- sudo apt upgrade

to run the software:

- sudo install jackd2
- sudo install git
- git clone https://github.com/mfvargo/rtjam_rust.git
- cd rtjam_rust
- make install

to build the software

- install rust - curl https://sh.rustup.rs -sSf | sh
- install libssl - sudo apt install libssl-dev
- install libjack-dev - sudo apt install jackd2 libjack-dev
- git clone https://github.com/mfvargo/rtjam_rust.git
- cd rtjam_rust
- cargo build etc ( or you can make )
