NATION=http://rtjam-nation.basscleftech.com/pi/rust
RTJAM_HOME=/home/pi/rtjam

all:
	cargo build --package rtjam_rust --example rtjam_broadcast --example rtjam_sound --release
	git rev-parse HEAD > target/release/examples/version.txt

clean:
	rm -rf target/release/examples/*

deploy: all
	scp -i ~/.ssh/rtjam.cer target/release/examples/rtjam_sound  ubuntu@rtjam-nation.basscleftech.com:/home/ubuntu/www/html/pi/rust
	scp -i ~/.ssh/rtjam.cer target/release/examples/rtjam_broadcast  ubuntu@rtjam-nation.basscleftech.com:/home/ubuntu/www/html/pi/rust
	scp -i ~/.ssh/rtjam.cer target/release/examples/version.txt  ubuntu@rtjam-nation.basscleftech.com:/home/ubuntu/www/html/pi/rust

install:
	mkdir -p $(RTJAM_HOME)
	wget -O $(RTJAM_HOME)/rtjam_sound $(NATION)/rtjam_sound
	chmod +x $(RTJAM_HOME)/rtjam_sound
	cp docs/pi-scripts/jackrun.bash $(RTJAM_HOME)
	chmod +x $(RTJAM_HOME)/jackrun.bash
	cp docs/pi-scripts/update-rtjam.bash $(RTJAM_HOME)
	chmod +x $(RTJAM_HOME)/update-rtjam.bash
	sudo cp docs/pi-scripts/rtjam-jack.service /etc/systemd/system
	sudo cp docs/pi-scripts/rtjam-sound.service /etc/systemd/system
	sudo systemctl daemon-reload
	sudo systemctl start rtjam-jack.service
	sudo systemctl start rtjam-sound.service

uninstall:
	sudo rm -f /etc/system.d/system/rtjam-jack.service
	sudo rm -f /etc/system.d/system/rtjam-sound.service
	sudo rm -rf $(RTJAM_HOME)
	sudo systemctl daemon-reload
