#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn read_ls_elf_poorly() {
        let ls_path = "testdata/ls";
        let bytes = fs::read(ls_path).unwrap();

        let exec_bytes = bytes.get(0x4000..0x13146).unwrap();

        println!("{:?}", exec_bytes);
    }
}
