
all:
	cargo build --package rtjam_rust --example rtjam_broadcast --example rtjam_sound --release
	git rev-parse HEAD > target/release/examples/version.txt

clean:
	cargo clean

deploy: all
	scp -i ~/.ssh/rtjam.cer target/release/examples/rtjam_sound  ubuntu@rtjam-nation.basscleftech.com:/home/ubuntu/www/html/pi/rust
	scp -i ~/.ssh/rtjam.cer target/release/examples/rtjam_broadcast  ubuntu@rtjam-nation.basscleftech.com:/home/ubuntu/www/html/pi/rust
	scp -i ~/.ssh/rtjam.cer target/release/examples/version.txt  ubuntu@rtjam-nation.basscleftech.com:/home/ubuntu/www/html/pi/rust
