use des::{cipher::{generic_array::GenericArray, BlockEncrypt as _, KeyInit as _}, Des};

pub fn des_enc(text: &str) -> String {
    fn pkcs7(text: &str) -> Vec<[u8; 8]> {
        assert!(text.len() > 7);
        assert!(text.len() < 17);
        let mut r = Vec::new();
        let pwd = text.as_bytes();
        let len = pwd.len();
        let batch = len / 8;
        let m = len % 8;
        for i in 0..batch {
            let mut a = [0u8; 8];
            a.copy_from_slice(&pwd[i * 8..8 + i * 8]);
            r.push(a);
        }
        let mut b = [0u8; 8];
        for i in 0..m {
            b[i] = pwd[8 * batch + i];
        }
        for item in b.iter_mut().skip(m) {
            *item = (8 - m) as u8;
        }
        r.push(b);
        // #[cfg(debug_assertions)]
        // println!("{r:?}");
        r
    }
    let key = b"u2oh6Vu^".to_owned();
    let key = GenericArray::from(key);
    let des = Des::new(&key);
    let mut data_block_enc = Vec::new();
    for block in pkcs7(text) {
        let mut block = GenericArray::from(block);
        des.encrypt_block(&mut block);
        let mut block = block.to_vec();
        data_block_enc.append(&mut block);
    }
    hex::encode(data_block_enc)
}
