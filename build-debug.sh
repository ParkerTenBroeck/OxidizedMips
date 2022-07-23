cargo build
mkdir -p ./mips/bin
cp ./target/mips/debug/binary ./mips/bin/com.o
cargo objcopy -- -O binary -I elf32-tradbigmips ./mips/bin/com.o ./mips/bin/tmp.bin
