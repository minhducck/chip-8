struct Processor {
    short opcode;
    char mut memory[4096];
}

struct CPU {
    pub registers: u8[16];
    pub IP: u16;
    pub stack: u8[48];

}

struct op_code {
    code: u8;
    pub fn execute();
}