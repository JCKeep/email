use actix_web::web::Buf;
use bytes::{BufMut, BytesMut};

fn main() {
    let mut buf = BytesMut::new();
    buf.put(&b"hello world"[..]);
    #[allow(overflowing_literals)]
    buf.put_i32_le(0x01020304);
    buf.put_i32(0x01020304);
    buf.put_f32(3.1_f32);

    let mut buf1 = buf.split_off("hello world".len());
    let v = buf1.as_mut();
    println!("{:?}", v);
    println!(
        "{:x} {:x} {}",
        buf1.get_i32_le(),
        buf1.get_i32(),
        buf1.get_f32()
    );

    let a = buf.freeze();
    let b = a.clone();
    println!("{:?}", a);
    assert_eq!(a.as_ptr() as u64, b.as_ptr() as u64);

    let s = String::from("hello\0\0world");
    println!("{} {}", s, s.len());
}
