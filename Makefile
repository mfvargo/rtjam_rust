NATION=http://rtjam-nation.com/pi/rust
RTJAM_HOME=/home/pi/rtjam

all:
	cargo build --package rtjam_rust --example rtjam_broadcast --example rtjam_sound --release
	git rev-parse HEAD > target/release/examples/version.txt

clean:
	rm -rf target/release/examples/*

deploy: all
	scp -i ~/.ssh/rtjam.cer target/release/examples/rtjam_sound  ubuntu@rtjam-nation.com:/home/ubuntu/www/html/pi/rust
	scp -i ~/.ssh/rtjam.cer target/release/examples/rtjam_broadcast  ubuntu@rtjam-nation.com:/home/ubuntu/www/html/pi/rust
	scp -i ~/.ssh/rtjam.cer target/release/examples/version.txt  ubuntu@rtjam-nation.com:/home/ubuntu/www/html/pi/rust

install-base:
	mkdir -p $(RTJAM_HOME)
	wget -O $(RTJAM_HOME)/rtjam_sound $(NATION)/rtjam_sound
	chmod +x $(RTJAM_HOME)/rtjam_sound
	wget -O $(RTJAM_HOME)/rtjam_broadcast $(NATION)/rtjam_sound
	chmod +x $(RTJAM_HOME)/rtjam_broadcast
	cp docs/pi-scripts/jackrun.bash $(RTJAM_HOME)
	chmod +x $(RTJAM_HOME)/jackrun.bash
	cp docs/pi-scripts/update-sound.bash $(RTJAM_HOME)
	chmod +x $(RTJAM_HOME)/update-sound.bash
	cp docs/pi-scripts/update-broadcast.bash $(RTJAM_HOME)
	chmod +x $(RTJAM_HOME)/update-broadcast.bash
	sudo cp docs/pi-scripts/rtjam-jack.service /etc/systemd/system
	sudo cp docs/pi-scripts/rtjam-sound.service /etc/systemd/system
	sudo cp docs/pi-scripts/rtjam-broadcast.service /etc/systemd/system
	sudo systemctl daemon-reload

install-sound: install-base
	sudo systemctl start rtjam-jack
	sudo systemctl start rtjam-sound
	sudo systemctl enable rtjam-jack
	sudo systemctl enable rtjam-sound

install-broadcast: install-base
	sudo systemctl start rtjam-broadcast
	sudo systemctl enable rtjam-broadcast

uninstall:
	sudo systemctl stop rtjam-broadcast
	sudo systemctl stop rtjam-sound
	sudo systemctl stop rtjam-jack
	sudo systemctl disable rtjam-broadcast
	sudo systemctl disable rtjam-sound
	sudo systemctl disable rtjam-jack
	sudo rm -f /etc/systemd/system/rtjam-jack.service
	sudo rm -f /etc/systemd/system/rtjam-sound.service
	sudo rm -f /etc/systemd/system/rtjam-broadcast.service
	sudo rm -rf $(RTJAM_HOME)
	sudo systemctl daemon-reload
