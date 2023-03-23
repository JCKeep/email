# EMAIL

SMTP与POP3服务器采用Rust高性能tokio异步框架, 提高服务器性能与并发量。

## SMTP

### 命令

|命令　　|　　    描述|
|------------|------------------|
|HELO　　  | 　  向服务器标识用户身份发送者能欺骗，说谎，但一般情况下服务器都能检测到。|
|MAIL　　　|　　　初始化邮件传输MAIL　FROM: <email address>|
|RCPT　　　|　　　标识单个的邮件接收人；常在MAIL命令后面可有多个RCPT TO: <email address>|
|DATA　　　|　　　在单个或多个RCPT命令后，表示所有的邮件接收人已标识，并初始化数据传输，以.结束。|
|VRFY　　　|　　　用于验证指定的用户/邮箱是否存在；由于安全方面的原因，服务器常禁止此命令|
|EXPN　　　|　　　验证给定的邮箱列表是否存在，扩充邮箱列表，也常被禁用|
|HELP　　  |　　　查询服务器支持什么命令|
|NOOP　　　| 　 　无操作，服务器应响应OK|
|QUIT　　　|　　　结束会话|
|RSET　　　|　　　重置会话，当前传输被取消|


### 响应码

|应答码|说明|
|-|-|
|501   |    参数格式错误|
|502   |    命令不可实现|
|503   |    错误的命令序列|
|504   |    命令参数不可实现|
|211   |    系统状态或系统帮助响应|
|214   |    帮助信息|
|220   |    服务就绪|
|221   |    服务关闭|
|421   |    服务未就绪，关闭传输信道|
|250   |    要求的邮件操作完成|
|251   |    用户非本地，将转发向＜forward-path＞|
|450   |    要求的邮件操作未完成，邮箱不可用|
|550   |    要求的邮件操作未完成，邮箱不可用|
|451   |    放弃要求的操作；处理过程中出错|
|551   |    用户非本地，请尝试＜forward-path＞|
|452   |    系统存储不足，要求的操作未执行|
|552   |    过量的存储分配，要求的操作未执行|
|553   |    邮箱名不可用，要求的操作未执行|
|354   |    开始邮件输入，以"."结束|
|554   |    操作失败|


### *Examples*

* `telnet smtp.qq.com 25`

```
root@DESKTOP-QVUCHRV:~# telnet smtp.qq.com 25
Trying 157.148.54.34...
Connected to smtp.qq.com.
Escape character is '^]'.
220 newxmesmtplogicsvrszc13.qq.com XMail Esmtp QQ Mail Server.
HELO localhost
250-newxmesmtplogicsvrszc13.qq.com-9.46.14.43-29816486
250-SIZE 73400320
250 OK
auth login
334 VXNlcm5hbWU6
MjQwNzAxODM3MUBxcS5jb20=
334 UGFzc3dvcmQ6
Zmhid3lzb2dhcGh5ZGlnYQ==
235 Authentication successful
MAIL FROM: <2407018371@qq.com>
250 OK
RCPT TO: <monster_t@foxmail.com>
250 OK
DATA
354 End data with <CR><LF>.<CR><LF>.
<p><em>hello world!</em></p>

.
250 OK: queued as.
QUIT
221 Bye.
Connection closed by foreign host.
```

## POP3

### 命令

|命令|描述|
|-|-|
|USER [username]|处理用户名|
|PASS [password]|处理用户密码|
|STAT|处理请求服务器发回关于邮箱的统计资料，如邮件总数和总字节数|
|LIST|处理返回邮件数量和每个邮件的大小|
|RETR|处理返回由参数标识的邮件的全部文本|
|DELE|处理服务器将由参数标识的邮件标记为删除，由quit命令执行|
|RSET|处理服务器将重置所有标记为删除的邮件，用于撤消DELE命令|
|NOOP|处理服务器返回一个肯定的响应|
|QUIT|终止会话|

### *Examples*

* `telnet pop.qq.com 110`

```
root@DESKTOP-QVUCHRV:~# telnet pop.qq.com 110
Trying 14.18.175.202...
Connected to pop.qq.com.
Escape character is '^]'.
+OK XMail POP3 Server v1.0 Service Ready(XMail v1.0)
user 2407018371
+OK
pass fhbwysogaphydiga
+OK
quit
+OK Bye
Connection closed by foreign host.
```

## TOKIO

* [tokio较为详细的文档](https://rust-book.junmajinlong.com/ch100/00.html)

```rust
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::mpsc,
};

#[tokio::main]
async fn main() {
    let server = TcpListener::bind("127.0.0.1:8888").await.unwrap();
    while let Ok((client_stream, client_addr)) = server.accept().await {
        println!("accept client: {}", client_addr);
        // 每接入一个客户端的连接请求，都分配一个子任务，
        // 如果客户端的并发数量不大，为每个客户端都分配一个thread，
        // 然后在thread中创建tokio runtime，处理起来会更方便
        tokio::spawn(async move {
            process_client(client_stream).await;
        });
    }
}

async fn process_client(client_stream: TcpStream) {
    let (client_reader, client_writer) = client_stream.into_split();
    let (msg_tx, msg_rx) = mpsc::channel::<String>(100);
  
  	// 从客户端读取的异步子任务
    let mut read_task = tokio::spawn(async move {
        read_from_client(client_reader, msg_tx).await;
    });

    // 向客户端写入的异步子任务
    let mut write_task = tokio::spawn(async move {
        write_to_client(client_writer, msg_rx).await;
    });

    // 无论是读任务还是写任务的终止，另一个任务都将没有继续存在的意义，因此都将另一个任务也终止
    if tokio::try_join!(&mut read_task, &mut write_task).is_err() {
        eprintln!("read_task/write_task terminated");
        read_task.abort();
        write_task.abort();
    };
}

/// 从客户端读取
async fn read_from_client(reader: OwnedReadHalf, msg_tx: mpsc::Sender<String>) {
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut buf = String::new();
    loop {
        match buf_reader.read_line(&mut buf).await {
            Err(_e) => {
                eprintln!("read from client error");
                break;
            }
            // 遇到了EOF
            Ok(0) => {
                println!("client closed");
                break;
            }
            Ok(n) => {
                // read_line()读取时会包含换行符，因此去除行尾换行符
                // 将buf.drain(。。)会将buf清空，下一次read_line读取的内容将从头填充而不是追加
                buf.pop();
                let content = buf.drain(..).as_str().to_string();
                println!("read {} bytes from client. content: {}", n, content);
                // 将内容发送给writer，让writer响应给客户端，
                // 如果无法发送给writer，继续从客户端读取内容将没有意义，因此break退出
                if msg_tx.send(content).await.is_err() {
                    eprintln!("receiver closed");
                    break;
                }
            }
        }
    }
}

/// 写给客户端
async fn write_to_client(writer: OwnedWriteHalf, mut msg_rx: mpsc::Receiver<String>) {
    let mut buf_writer = tokio::io::BufWriter::new(writer);
    while let Some(mut str) = msg_rx.recv().await {
        str.push('\n');
        if let Err(e) = buf_writer.write_all(str.as_bytes()).await {
            eprintln!("write to client failed: {}", e);
            break;
        }
    }
}
```
