// TODO: Add a hello_world for 32-bits and 16-bits if possible.
mod prefix;
mod opcode;
mod modrm;
mod imm;
mod reg;
mod reader;
mod dis;


#[cfg(test)]
mod tests {
    use std::fs;
    use crate::reader::Reader;

    #[test]
    fn read_ls_elf_poorly() {
        let ls_path = "testdata/ls";
        let bytes = fs::read(ls_path).unwrap();

        let entry_point = 0x6ab0;
        let exec_bytes = bytes.get(0x4000..0x13146).unwrap();

        let first_20_bytes = bytes.get(entry_point..entry_point + 20).unwrap();
        let actual_first_20_bytes = 
            vec![0xf3, 0x0f, 0x1e, 0xfa, 0x31, 0xed, 0x49, 0x89, 0xd1, 0x5e, 0x48, 0x89, 0xe2, 
                    0x48, 0x83, 0xe4, 0xf0, 0x50, 0x54, 0x45];


        assert!(actual_first_20_bytes == first_20_bytes);
        
    }

    #[test]
    fn test_reader() {
        let ls_path = "testdata/ls";
        let bytes = fs::read(ls_path).unwrap();

        let mut reader = Reader::from_vec(bytes);

        for i in 0..3 {
            let number = reader.read::<u64>().unwrap();
            match i {
                0 => assert!(number == 282584257676671),
                1 => assert!(number == 0),
                2 => assert!(number == 4299030531),
                _ => panic!("Should not have value {i}"),
            }
        }
    }
}
