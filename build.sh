cargo build --release
mkdir -p ./mips/bin
cp ./target/mips/release/mips_template ./mips/bin/com.o
mips-linux-gnu-objcopy -O binary -I elf32-tradbigmips ./mips/bin/com.o ./mips/bin/tmp.bin
