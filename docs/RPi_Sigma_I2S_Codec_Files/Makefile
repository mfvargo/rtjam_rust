
all:
	dtc -@ -H epapr -O dtb -o sigma-i2s-codec_RPi5.dtbo -Wno-unit_address_vs_reg sigma-i2s-codec_RPi5.dts
	dtc -@ -H epapr -O dtb -o sigma-i2s-codec_RPi4.dtbo -Wno-unit_address_vs_reg sigma-i2s-codec_RPi4.dts

clean:
	rm -f sigma-i2s-codec_RPi5.dtbo sigma-i2s-codec_RPi4.dtbo