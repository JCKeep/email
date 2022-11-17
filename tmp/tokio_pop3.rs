use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    let connection = TcpStream::connect("pop.qq.com:110").await.unwrap();
    let (mut rd, mut wr) = io::split(connection);
    let mut buf = vec![0; 1024];

    {
        let n = rd.read(&mut buf).await.unwrap();
        print!("{}", String::from_utf8_lossy(&buf[..n]));
        wr.write(b"user 2407018371@qq.com\r\n").await.unwrap();

        let n = rd.read(&mut buf).await.unwrap();
        print!("{}", String::from_utf8_lossy(&buf[..n]));
        wr.write(b"pass fhbwysogaphydiga\r\n").await.unwrap();

        let n = rd.read(&mut buf).await.unwrap();
        print!("{}", String::from_utf8_lossy(&buf[..n]));
        wr.write(b"list\r\n").await.unwrap();

        loop {
            match rd.read(&mut buf).await {
                Ok(n) if n == 0 => {
                    break;
                }
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    print!("{}", s);
                    if s.ends_with(".\r\n") {
                        break;
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
        // wr.write(b"retr 37\r\n").await.unwrap();
        // loop {
        //     match rd.read(&mut buf).await {
        //         Ok(n) if n == 0 => {
        //             break;
        //         }
        //         Ok(n) => {
        //             let s = String::from_utf8_lossy(&buf[..n]);
        //             print!("{}", s);
        //             if s.ends_with(".\r\n") {
        //                 break;
        //             }
        //         }
        //         Err(_) => {
        //             break;
        //         }
        //     }
        // }
        // wr.write(b"top 91 1\r\n").await.unwrap();
        // loop {
        //     match rd.read(&mut buf).await {
        //         Ok(n) if n == 0 => {
        //             break;
        //         }
        //         Ok(n) => {
        //             let s = String::from_utf8_lossy(&buf[..n]);
        //             print!("{}", s);
        //             if s.ends_with(".\r\n") {
        //                 break;
        //             }
        //         }
        //         Err(_) => {
        //             break;
        //         }
        //     }
        // }
        wr.write(b"dele 94\r\n").await.unwrap();
        let n = rd.read(&mut buf).await.unwrap();
        print!("{}", String::from_utf8_lossy(&buf[..n]));
        wr.write(b"quit\r\n").await.unwrap();
    }
}
