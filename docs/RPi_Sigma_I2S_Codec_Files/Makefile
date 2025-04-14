# This makefile will produce all the various dtbo overlay files for the custom rtjam hardware.  There are two
# main types, those for the pi 4 and those for the pi 5. The main difference for the two main types is the 
# clock configuration.  i2s for the 4 and i2s_clock_consumer for the 5.  The other differences are just in the 
# name of the driver which will allow the u/x to present the correct volume controls depending on what knobs
# are present on the hardware.

all: prepare
	dtc -@ -Hepapr -I dts -O dtb -o rtjam4-0knob.dtbo -Wno-unit_address_vs_reg rtjam4-0knob.dts
	dtc -@ -Hepapr -I dts -O dtb -o rtjam4-1knob.dtbo -Wno-unit_address_vs_reg rtjam4-1knob.dts
	dtc -@ -Hepapr -I dts -O dtb -o rtjam4-3knob.dtbo -Wno-unit_address_vs_reg rtjam4-3knob.dts
	dtc -@ -Hepapr -I dts -O dtb -o rtjam5-0knob.dtbo -Wno-unit_address_vs_reg rtjam5-0knob.dts
	dtc -@ -Hepapr -I dts -O dtb -o rtjam5-1knob.dtbo -Wno-unit_address_vs_reg rtjam5-1knob.dts
	dtc -@ -Hepapr -I dts -O dtb -o rtjam5-3knob.dtbo -Wno-unit_address_vs_reg rtjam5-3knob.dts

clean:
	rm -f *.dtbo
	rm -f rtjam4-0knob.dts rtjam4-1knob.dts rtjam4-3knob.dts
	rm -f rtjam5-0knob.dts rtjam5-1knob.dts rtjam5-3knob.dts

prepare:
	sed -e 's/DRIVER_NAME_HERE/rtjam4-0knob/g' -e 's/CLOCK_CONFIG_HERE/i2s/g' source_file.dts > rtjam4-0knob.dts
	sed -e 's/DRIVER_NAME_HERE/rtjam4-1knob/g' -e 's/CLOCK_CONFIG_HERE/i2s/g' source_file.dts > rtjam4-1knob.dts
	sed -e 's/DRIVER_NAME_HERE/rtjam4-3knob/g' -e 's/CLOCK_CONFIG_HERE/i2s/g' source_file.dts > rtjam4-3knob.dts
	sed -e 's/DRIVER_NAME_HERE/rtjam5-0knob/g' -e 's/CLOCK_CONFIG_HERE/i2s_clk_consumer/g' source_file.dts > rtjam5-0knob.dts
	sed -e 's/DRIVER_NAME_HERE/rtjam5-1knob/g' -e 's/CLOCK_CONFIG_HERE/i2s_clk_consumer/g' source_file.dts > rtjam5-1knob.dts
	sed -e 's/DRIVER_NAME_HERE/rtjam5-3knob/g' -e 's/CLOCK_CONFIG_HERE/i2s_clk_consumer/g' source_file.dts > rtjam5-3knob.dts
