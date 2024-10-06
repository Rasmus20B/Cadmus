use crate::{dbcharconv::encode_text, encoding_util::{fm_string_encrypt, get_int, get_path_int, put_int, put_path_int}, FmpFileRepresentation};

use std::{fs::{write, File}, io::Read, path::Path};
use super::header::HEADER_INIT;
use crate::encoding_util;
use crate::dbcharconv;

pub struct Assembler<'a> {
    idx: usize,
    local_idx: u16,
    buffer: Vec<u8>,
    file: &'a FmpFileRepresentation,
}

impl<'a> Assembler<'a> {
    pub fn new(input: &'a FmpFileRepresentation) -> Self {
        Self {
            idx: 0,
            local_idx: 0,
            buffer: vec![],
            file: input,
        }
    }

    fn emit_noop(&mut self) {
        self.buffer[self.idx] = 0x80;
        self.idx += 1;
    }


    /* Only used when value is not a known datatype */
    pub fn calc_small_kv_ins(&self, val: &[u8]) -> u8 {
        let code = ((val.len() + 1).div_ceil(2)) as u8;
        if code > 0x5 {
            return 0x6;
        }

        return code;
        ((val.len() + 1).div_ceil(2)) as u8
    }

    fn emit_simple_data_1b(&mut self, data: &[u8]) {
        self.buffer[self.idx] = 0x1b;
        self.idx += 1;
        self.buffer[self.idx] = data.len() as u8;
        self.idx += 1;
        self.buffer.splice(self.idx..self.idx+data.len(), data.to_vec());
        self.idx += data.len() + 2 * (0x1b as usize - 0x19);
    }

    fn emit_simple_data(&mut self, data: &[u8]) {

    }

    fn emit_long_kv(&mut self, key: &[u8], val: &[u8]) {

        let code = 0x1e;
        self.buffer[self.idx] = code;
        self.idx += 1;
        self.buffer[self.idx] = key.len() as u8;
        self.idx += 1;
        // let db_encoding : Vec<(u8, u8)> = encoding
        //     .chunks_exact(2)
        //     .map(|chunk| (chunk[0], chunk[1]))
        //     .collect();
        self.buffer.splice(self.idx..self.idx+key.len(), key.to_vec());
        self.idx += key.len();
        self.buffer[self.idx] = val.len() as u8;
        self.idx += 1;
        self.buffer.splice(self.idx..self.idx+val.len(), val.to_vec());
        self.idx += val.len();
    }
    
    fn emit_simple_kv_e(&mut self, key: u32, val: &[u8]) {

        self.buffer[self.idx] = 0x0e;
        self.idx += 1;
        let n = put_path_int(key);
        let n_len = n.len();
        println!("Pushing {:?}", n);
        self.buffer.splice(self.idx..self.idx+n_len, n);
        self.idx += n_len;
        self.buffer[self.idx] = val.len() as u8;
        self.idx += 1;
        self.buffer.splice(self.idx..self.idx+val.len(), val.to_vec());
        self.idx += val.len();
    }

    fn emit_simple_kv(&mut self, key: u8, val: &[u8]) {
        let ins = match key {
            16 => { 0x6 }
            216 => { 0x6 }
            _ => { self.calc_small_kv_ins(val) }
        };

        self.buffer[self.idx] = ins;
        self.idx += 1;


        match ins {
            0x01 | 0x02 | 0x03 | 0x04 | 0x05 => {
                self.buffer[self.idx] = key as u8;
                self.idx += 1;
                self.buffer.splice(self.idx..self.idx+val.len(), val.to_vec());
                self.idx += val.len();
            }
            0x6 => {
                self.buffer[self.idx] = key as u8;
                println!("Ins: {} -> key {} :: {:?} :: len {}", ins, key, val, val.len() as u8);
                self.idx += 1;
                self.buffer[self.idx] = val.len() as u8;
                self.idx += 1;
                self.buffer.splice(self.idx..self.idx+val.len(), val.to_vec());
                self.idx += val.len();
            }
            _ => {}
        }
    }

    fn pop_directory(&mut self) {
        self.buffer[self.idx] = 0x40;
        self.idx += 1;
    }

    fn push_directory(&mut self, dir: u32) {
        let d = put_path_int(dir);
        let ins = match d.len() {
            1 => { 0x20 }
            2 => { 0x28 }
            _ => { 0x48 }
        };

        println!("Ins: {:x}, dir: {:?}", ins, d);

        self.buffer[self.idx] = ins;
        self.idx += 1;
        self.buffer.splice(self.idx..self.idx + d.len(), d.clone());
        self.idx += d.len();
        self.emit_noop();
    }

    pub fn append_blank_chunk(&mut self) {
        self.buffer.resize(self.buffer.len() + 4096, 0);
        self.idx = self.buffer.len() - 4095;
        self.local_idx = 0;
    }

    pub fn emit_init_blobs(&mut self) {
        self.push_directory(2);
        self.emit_simple_kv(3, &[120, 104, 106, 116, 107, 120]);
        self.emit_simple_kv(5, &[4, 184]);
        self.emit_simple_kv(6, &[1, 10, 4, 179, 4, 115, 8, 179, 0, 19, 0, 0, 3, 124, 4, 0, 18, 5, 66, 128, 2, 0, 0, 0, 0, 66, 144, 0, 0, 0, 8, 2, 0, 0, 0, 15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        self.emit_simple_kv(8, &[78, 152, 78, 152, 78, 152, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23, 10, 10, 40, 53, 122, 104, 106, 116, 107, 116, 104, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 8, 107, 122, 107, 110, 116, 107, 116, 107, 0, 0, 0, 21, 6, 107, 122, 107, 110, 116, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        self.emit_simple_kv(9, &[0, 0, 0, 0, 11, 251, 83, 44, 0, 0, 0, 0, 11, 251, 83, 44, 0, 0, 0, 0, 11, 251, 83, 44, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 23, 9, 104, 106, 107, 106, 104, 106, 104, 106, 110, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        self.emit_simple_kv(11, &[0, 8, 7, 60, 51, 54, 63, 55, 59, 57, 98, 65, 50, 218, 23, 59, 57, 51, 52, 46, 53, 41, 50, 122, 18, 30, 218, 27, 42, 42, 54, 51, 57, 59, 46, 51, 53, 52, 41, 218, 28, 51, 54, 63, 23, 59, 49, 63, 40, 122, 10, 40, 53, 116, 59, 42, 42, 218, 25, 53, 52, 46, 63, 52, 46, 41, 218, 8, 63, 41, 53, 47, 40, 57, 63, 41, 218, 19, 52, 41, 46, 59, 54, 54, 63, 40, 218, 31, 34, 46, 63, 52, 41, 51, 53, 52, 41, 218, 30, 51, 57, 46, 51, 53, 52, 59, 40, 51, 63, 41, 218, 12, 47, 49, 63, 52, 61, 54, 41, 50, 116, 55, 42, 40, 7, 60, 51, 54, 63, 55, 59, 57, 90, 65, 50, 218, 23, 59, 57, 51, 52, 46, 53, 41, 50, 122, 18, 30, 218, 15, 41, 63, 40, 41, 218, 55, 63, 40, 51, 55, 59, 49, 218, 22, 51, 56, 40, 59, 40, 35, 218, 27, 42, 42, 54, 51, 57, 59, 46, 51, 53, 52, 122, 9, 47, 42, 42, 53, 40, 46, 218, 28, 51, 54, 63, 23, 59, 49, 63, 40, 218, 31, 34, 46, 63, 52, 41, 51, 53, 52, 41, 218, 30, 51, 57, 46, 51, 53, 52, 59, 40, 51, 63, 41, 218, 8, 15, 41, 63, 40, 116, 47, 42, 40]);
        self.emit_simple_kv(24, &[0, 0, 4, 3, 1, 1, 0, 0, 2, 1, 2, 0, 1, 0, 0, 0, 1, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 0, 1, 0, 0]);
        self.pop_directory();
    }


    pub fn emit_table_metadata(&mut self) {
        self.push_directory(3);
        self.push_directory(16);
        self.push_directory(1);
        self.push_directory(1);

        for t in &self.file.tables {
            let mut encoding = dbcharconv::encode_text(&t.1.table_name);
            encoding.push(0x0);
            encoding.push(0x0);
            encoding.push(0x0);
            /* TODO: second argument shoudl not be hard coded in this way */
            self.emit_long_kv(&encoding, &[2, 128, (*t.0 as u8)]);
        }

        self.pop_directory();
        self.push_directory(3);

        for t in &self.file.tables {
            let key = put_int(*t.0);
            println!("pushing {:?}", (*t.0 as u32).to_be_bytes());
            self.emit_long_kv(&key, &(*t.0 as u32).to_be_bytes());
        }
        self.pop_directory();
        self.emit_simple_kv(4, &put_int(self.file.tables.len()));
        self.emit_simple_kv(216, &fm_string_encrypt("hello".to_string()));
        self.emit_simple_kv_e(129, &vec![59, 62, 55, 51, 52]);
        self.emit_simple_kv_e(130, &vec![59, 62, 55, 51, 52]);

        self.push_directory(5);

        for t in &self.file.tables {
            self.push_directory(128 + *t.0 as u32);
            self.pop_directory();
        }
        self.pop_directory();
        self.pop_directory();
        self.pop_directory();
    }

    pub fn emit_relationship_data(&mut self) {
        self.push_directory(17);
        self.push_directory(1);
        self.emit_simple_kv(0, &[3, 208, 0, 9]);
        self.push_directory(1);

        for to in &self.file.table_occurrences {
            let name = &encode_text(&(to.1.table_occurence_name.clone() + "\0\0"));
            self.emit_simple_data_1b(&name);
        }

        self.pop_directory();
        self.push_directory(3);
        for to in &self.file.table_occurrences {
            self.emit_simple_data_1b(&put_int(*to.0));
        }

        self.pop_directory();
        self.push_directory(8);
        self.pop_directory();
        self.pop_directory();
        self.pop_directory();
        self.pop_directory();
    }

    pub fn emit_layout_data(&mut self) {
        self.push_directory(4);
        self.push_directory(1);
        self.emit_simple_kv(0, &[1, self.file.layouts.len() as u8]);
        self.push_directory(5);
        self.pop_directory();
        self.pop_directory();
        self.pop_directory();
    }
    
    pub fn emit_theme_data(&mut self) {
        self.push_directory(6);
        self.pop_directory();
    }

    pub fn emit_script_data(&mut self) {
        self.push_directory(17);
        self.push_directory(1);
        self.emit_simple_kv(0, &[1, self.file.scripts.len() as u8]);

        self.push_directory(1);

        for script in &self.file.scripts {
            self.emit_simple_data(&encode_text(&script.1.script_name));
        }
        self.pop_directory();
        self.pop_directory();
        self.pop_directory();
    }

    pub fn emit_security_data(&mut self) {
        self.push_directory(23);

        self.pop_directory();
    }

    pub fn emit_value_list_data(&mut self) {
        self.push_directory(33);
        self.pop_directory();
    }

    pub fn emit_font_data(&mut self) {
        self.push_directory(25);
        self.pop_directory();
    }

    pub fn emit_toolbar_data(&mut self) {
        self.push_directory(65);
        self.pop_directory();
    }

    pub fn emit_table_data(&mut self) {

    }


    pub fn assemble_fmp12(&mut self, schema: &FmpFileRepresentation) {

        self.buffer.extend(HEADER_INIT);

        let name_header = schema.name.clone() + "/";
        self.buffer.splice(3082..3082+name_header.len(), name_header.as_bytes().to_vec());
        // println!("{} == {:?}", name_header, &self.buffer[3082..3082+name_header.len()]);

        self.append_blank_chunk();
        self.buffer.splice((self.idx+8) as usize..(self.idx+12) as usize, 4_u32.to_be_bytes());
        self.append_blank_chunk();
        self.buffer.splice((self.idx+8) as usize..(self.idx+12) as usize, 3_u32.to_be_bytes());
        self.idx += 20;
        self.emit_noop();
        self.emit_init_blobs();
        self.emit_table_metadata();
        self.emit_relationship_data();
        self.emit_layout_data();
        self.emit_theme_data();
        self.emit_script_data();
        self.emit_security_data();
        self.emit_theme_data();
        self.emit_font_data();
        self.emit_toolbar_data();
        self.append_blank_chunk();
        self.buffer.splice((self.idx+8) as usize..(self.idx+12) as usize, 4_u32.to_be_bytes());
        self.idx += 20;
        self.append_blank_chunk();
        self.buffer.splice((self.idx+8) as usize..(self.idx+12) as usize, 0_u32.to_be_bytes());
    }

    pub fn emit_assembly(&self, filename: &str) {
        write(format!("{}.fmp12", filename), &self.buffer).expect("unable to write to file.");
    }
}

#[cfg(test)]
mod tests {
    use super::Assembler;
    use crate::FmpFileRepresentation;
    #[test]
    fn kv_instruction_test() {
        let tmp = FmpFileRepresentation::new();
        let assembler = Assembler::new(&tmp);
        assert_eq!(0x4, assembler.calc_small_kv_ins(&[120, 104, 106, 116, 107, 120]));
        assert_eq!(0x2, assembler.calc_small_kv_ins(&[4, 184]));
        assert_eq!(0x5, assembler.calc_small_kv_ins(&[0, 0, 0, 4, 56, 1, 64, 1]));
        assert_eq!(0x6, assembler.calc_small_kv_ins(&[78, 152, 78, 152, 78, 152, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                23, 10, 10, 40, 53, 122, 104, 106, 116, 107, 116, 104, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 2, 8, 107, 122, 107, 110, 116, 107, 116,
                107, 0, 0, 0, 21, 6, 107, 122, 107, 110, 116, 111, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0]));
    }
}

