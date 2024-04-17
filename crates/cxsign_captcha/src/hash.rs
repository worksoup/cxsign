use rand::Rng;

use cxsign_obfuscate::_f1;

// 混淆代码，不过仅仅在仓库中不可见。
// 如果你使用 IDE 或者对 rust 比较
// 熟悉的话应该很容易看到源码。
_f1!();
// let mut s = String::new();
// for i in (0..0x20 * a.len()).step_by(0x8) {
//     s.push(((a[i >> 5] >> (i % 0x20 & 0xff)) & 0xff) as u8 as char)
// }
// s
#[inline(always)]
fn to_bytes(a: [u32; 4]) -> [u8; 16] {
    unsafe { std::mem::transmute(a) }
}
#[inline(always)]
fn pre_hash(a: &str) -> Vec<u32> {
    let mut array = vec![0_u32; (a.len() >> 2) + 1];
    for i in 0..a.len() {
        let index = i >> 0x02;
        array[index] |= (0xff & a.as_bytes()[i] as u32) << ((i * 8) % 0x20);
    }
    array
}
// 绝对会 panic.
fn _unused(a: &str, b: &str) -> [u8; 16] {
    let mut s = pre_hash(a);
    if 0x10 < s.len() {
        s = hash_(s, 0x08 * a.len()).to_vec();
    }
    let mut l = vec![0; 0x0f];
    let mut m = vec![0; 0x0f];
    for i in 0..0x10 {
        l[i] = 0x36363636 ^ s[i];
        m[i] = 0x5c5c5c5c ^ s[i];
    }
    l.append(&mut pre_hash(b));
    let a = hash_(l, 0x200 + 0x08 * b.len());
    m.append(&mut a.to_vec());
    to_bytes(hash_(m, 0x280))
}

pub(crate) fn hash(a: &str) -> [u8; 16] {
    to_bytes(hash_(pre_hash(a), a.len()))
}

// let mut s = String::new();
// for c in a {
// s.push("0123456789abcdef".as_bytes()[((c >> 4) & 0x0f_u8) as usize] as char);
// s.push("0123456789abcdef".as_bytes()[(c & 0x0f_u8) as usize] as char);
// }
// s
#[inline(always)]
pub(crate) fn encode(a: [u8; 16]) -> String {
    hex::encode(a)
}
pub(crate) fn uuid() -> String {
    let mut v = [0; 0x24];
    let hex: [u8; 16] = *b"0123456789abcdef";
    v = v.map(|_| {
        let index = rand::thread_rng().gen_range(0x00..0x10) as usize;
        hex[index]
    });
    v[0x0e] = b'4';
    let i =
        (0x3 & if (b'0'..=b'9').contains(&v[0x13]) {
            v[0x13]
        } else {
            0
        }) | 0x8;
    v[0x13] = hex[i as usize];
    for i in [0x08, 0x0d, 0x12, 0x17] {
        v[i] = b'-';
    }
    v.map(|a| a as char).iter().collect()
}

#[cfg(test)]
mod tests {
    use crate::hash::{encode, hash_, pre_hash, to_bytes, uuid};

    // #[test]
    // fn it_works() {
    //     print!("[");
    //     for k in C {
    //         let k = k as u32;
    //         print!("0x{k:02x}, ");
    //     }
    //     println!("]");
    // }
    #[test]
    fn f() {
        print!("[");
        let mut i = 0;
        for _ in 0..16 {
            print!("0x{i:02x}, ");
            i += 1;
        }
        i = 1;
        for _ in 0..16 {
            print!("0x{i:02x}, ");
            i = (i + 5) % 16;
        }
        i = 5;
        for _ in 0..16 {
            print!("0x{i:02x}, ");
            i = (i + 3) % 16;
        }
        i = 0;
        for _ in 0..16 {
            print!("0x{i:02x}, ");
            i = (i + 7) % 16;
        }
        println!("]");
    }
    #[test]
    fn pre_hash_test() {
        let k = "12121212";
        println!("{:?}", pre_hash(k));
        let k = hash_(pre_hash(k), k.len());
        println!("{:?}", 0x0e + (((32 + 64) >> 9) << 4));
        println!("{:?}", 128 << 32 % 32);
        println!("{:?}", to_bytes(k));
        println!("{:?}", encode(to_bytes(k)));
        let k = k.map(|a| a as i32);
        println!("{:?}", k);
        let u = uuid();
        println!("{:?}", u);
    }
}
