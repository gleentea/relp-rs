use tokio::io::{AsyncReadExt,AsyncWriteExt};
use crate::{frame::RelpFrame, common::{RelpError, RelpCommand, RELP_PROTOCOL_VERSION, RELP_SUPPORTED_COMMAND, RELPRS_SOFTWARE_NAME, RELPRS_SOFTWARE_VERSION}};


#[derive(Debug)]
pub struct RelpSession<S: AsyncReadExt + AsyncWriteExt> {
    stream: S,
    ready: bool,
    buf: Vec<u8>,
    closed: bool,
}

impl<S:AsyncReadExt + AsyncWriteExt + std::marker::Unpin> RelpSession<S> {
    pub fn new(stream: S) -> RelpSession<S> {
        RelpSession{stream, ready:false, buf: Vec::new(), closed: false}
    }

    pub fn is_open(&self) -> bool {
        ! self.closed
    }

    pub async fn open(&mut self) -> Result<bool,RelpError> {
        if self.is_ready() {
            return Err(RelpError::AlreadyOpened)
        }
        let mut frame = RelpFrame::default();
        match self.get_frame(&mut frame).await {
            Ok(_) => true,
            Err(e) => {
                let nack = frame.nack(&"ERR".to_string());
                match self.stream.write(nack.as_bytes()).await {
                    Ok(_) => true,
                    Err(_e) => return Err(RelpError::UnknownError)
                };
                return Err(e)
            }
        };
        match frame.cmd {
            RelpCommand::OPEN => true,
            _ => {
                let nack = frame.nack(&"ERR".to_string());
                let _ = self.stream.write(nack.as_bytes()).await;
                return Err(RelpError::InvalidCommand)
            }
        };
        let data = format!("OK\nrelp_version={}\nrelp_software={}_{}\ncommands={}",
                                    RELP_PROTOCOL_VERSION,
                                    RELPRS_SOFTWARE_NAME,
                                    RELPRS_SOFTWARE_VERSION,
                                    RELP_SUPPORTED_COMMAND);
        let ack = frame.ack(&data.to_string());
        let _ = self.stream.write(ack.as_bytes()).await;
        self.ready = true;
        Ok(true)
    }

    async fn pull_buff(&mut self) -> Result<bool,RelpError> {
        let mut localbuf = [0u8;1024];
        let n = match self.stream.read(&mut localbuf).await {
            Ok(n) => match n {
                    0 => {
                        self.closed = true;
                        return Ok(false)
                    },
                    n => {
                        n
                    },
                },
                Err(e) => {
                    println!("{:?}",e);
                    return Err(RelpError::StreamReadError)
                }
        };
        self.buf.append(&mut localbuf[0..n].to_vec());
        Ok(true)
    }

    pub async fn get_frame(&mut self, frame: &mut RelpFrame) -> Result<bool,RelpError> {
        loop {
            if self.buf.len() < RelpFrame::MINIMUM_SIZE {
                match self.pull_buff().await? {
                        true => true,
                        false => return Ok(false),

                };
            }

            let nreaded = match RelpFrame::from_vec(&self.buf, frame) {
                Ok(n) => match n {
                    Some(n) => n,
                    None => {
                        match self.pull_buff().await? {
                            true => true,
                            false => return Ok(false)
                        };
                        continue
                    },
                },
                Err(e) => return Err(e)
            };
            self.buf.drain(0..nreaded);
            break;
        }
        return Ok(true);
    }

    pub async fn ack(&mut self,frame: &RelpFrame) {
        let res = frame.ack(&"OK".to_string());
        self.stream.write(res.as_bytes()).await;
    }

    pub async fn nack(&mut self, frame: &RelpFrame) {
        let res = frame.nack(&"ERR".to_string());
        self.stream.write(res.as_bytes()).await;
    }

    pub fn is_ready(&self)-> bool {
        self.ready
    }
}


#[cfg(test)]
mod tests{
    const START_RSP: &str = "1 rsp 92 200 OK
relp_version=0
relp_software=librelp,1.5.0,http://librelp.adiscon.com
commands=syslog
";
}