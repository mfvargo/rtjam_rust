NATION=http://rtjam-nation.com/pi/rust-2
RTJAM_HOME=/home/pi/rtjam

all:
	cargo build --package rtjam_rust --examples --release
	cargo build --examples --release
	cargo doc --release --no-deps
	git rev-parse HEAD > target/release/examples/version.txt

drivers:
	make all -C docs/RPi_Sigma_I2S_Codec_Files

test:
	cargo test -- --test-threads=1

clean:
	cargo clean
	make clean -C docs/RPi_Sigma_I2S_Codec_Files

debug:
	cargo build --package rtjam_rust --example rtjam_broadcast --example rtjam_sound --example r2_2_wave --example rt_2_csv

# note that this deploy script make sense only on the pi build machine (pi@pi-5 from /etc/hosts)
deploy: all
	scp target/release/examples/rtjam_sound  mfvargo@rtjam-nation:/home/mfvargo/pi/rust-2
	scp target/release/examples/rtjam_broadcast  mfvargo@rtjam-nation:/home/mfvargo/pi/rust-2
	scp target/release/examples/version.txt  mfvargo@rtjam-nation:/home/mfvargo/pi/rust-2

install-base:
	mkdir -p $(RTJAM_HOME)
	mkdir -p $(RTJAM_HOME)/recs
	wget -O $(RTJAM_HOME)/rtjam_sound $(NATION)/rtjam_sound
	chmod +x $(RTJAM_HOME)/rtjam_sound
	wget -O $(RTJAM_HOME)/rtjam_broadcast $(NATION)/rtjam_broadcast
	chmod +x $(RTJAM_HOME)/rtjam_broadcast
	cp docs/pi-scripts/jackrun.bash $(RTJAM_HOME)
	chmod +x $(RTJAM_HOME)/jackrun.bash
	cp docs/pi-scripts/bcastrun.bash $(RTJAM_HOME)
	chmod +x $(RTJAM_HOME)/bcastrun.bash
	sudo cp docs/pi-scripts/rtjam-jack.service /etc/systemd/system
	sudo cp docs/pi-scripts/rtjam-broadcast.service /etc/systemd/system
	sudo systemctl daemon-reload

install-sound: install-base
	sudo systemctl start rtjam-jack
	sudo systemctl enable rtjam-jack

install-broadcast: install-base
	sudo systemctl start rtjam-broadcast
	sudo systemctl enable rtjam-broadcast

uninstall:
	sudo systemctl stop rtjam-broadcast
	sudo systemctl stop rtjam-jack
	sudo systemctl disable rtjam-broadcast
	sudo systemctl disable rtjam-jack
	sudo rm -f /etc/systemd/system/rtjam-jack.service
	sudo rm -f /etc/systemd/system/rtjam-broadcast.service
	sudo rm -rf $(RTJAM_HOME)
	sudo systemctl daemon-reload
