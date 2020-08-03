pub fn write_rom_to_mem(rom: Vec<u8>) -> [u8; 0x1FFF]{
    
    let mut mem: [u8; 0x1FFF] = [0; 0x1FFF];
    let mut mi :usize = 0x1000;
    let mut ri :usize = 0x0;

    while mi < 0x1FFF {
        mem[mi] = rom[ri];
        mi += 1;
        ri += 1;
    }

    return mem;
}