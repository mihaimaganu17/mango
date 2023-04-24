// TODO: Add a hello_world for 32-bits and 16-bits if possible.
mod prefix;
mod opcode;
mod modrm;
mod imm;

pub struct Displacement;
pub struct Immediate;

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn read_ls_elf_poorly() {
        let ls_path = "testdata/ls";
        let bytes = fs::read(ls_path).unwrap();

        let entry_point = 0x6ab0;
        let exec_bytes = bytes.get(0x4000..0x13146).unwrap();

        let first_20_bytes = bytes.get(entry_point..entry_point + 20).unwrap();

        println!("{:02x?}", first_20_bytes);
    }
}
