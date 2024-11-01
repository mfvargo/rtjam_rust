# Config the unit for OS loading
- unplug the 9V power
- connect pins 2-3 on jumper labeld USB on the carrier board
- connect the usb port to the compute which will load the software
- Get the usbboot code for the pi compute module and run rpiboot so it shows up as a usb drive (see https://github.com/raspberrypi/usbboot/tree/master
)
- Use the pi imager or other app to put the 64 bit os lite (no graphic u/x)

make sure pi os is up to date

```
sudo apt update
sudo apt upgrade
```

install jack
```
sudo apt install jackd2
```
