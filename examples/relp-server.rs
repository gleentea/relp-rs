

use clap::{Parser, arg};
use relp_rs::{session::RelpSession,frame::RelpFrame};
use tokio::{net::TcpListener};

#[derive(Parser,Debug)]
struct Cli {
    #[arg(short='p', long="port", default_value="10000")]
    port: u16,
    #[arg(short='b', long="bind", default_value="localhost")]
    bind: String,
}

#[tokio::main]
async fn main() {
    println!("** relp-rs example **");
    let args = Cli::parse();
    let addr: (String,u16) = (args.bind, args.port);

    println!("listening => {:?}",&addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        println!("[@] wait for incoming connection");
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut session = Box::new(RelpSession::new(socket));
            println!("[!] starting session open...");
            match session.open().await {
                Ok(r) => match r {
                    false => {
                        println!("[!] session open failed!");
                        return;
                    },
                    _ => true,
                },
                Err(_) => return
            };
            println!("[+] relp session opend.");
            while session.is_open() {
                let mut frame = RelpFrame::default();

                let r = session.get_frame(&mut frame).await;
                match r {
                    Ok(n) => match n {
                        true => {
                            println!("[-] {:?}", frame);
                            session.ack(&frame).await;
                        },
                        false => break,
                    },
                    Err(_e) => {
                        println!("{:?}",_e);
                        break},
                };
            }
            println!("[!] session closed.");
        });
    }
}