 use crate::{common::{RelpCommand, RelpStatus, RelpParserResult}, parser::get_frame};

#[derive(Debug,PartialEq,Default)]
pub struct RelpFrame {
    txnr: u32,
    pub cmd: RelpCommand,
    datalen: usize,
    pub data: String,
}

impl RelpFrame {
    pub const MINIMUM_SIZE: usize = 7;

    pub fn new(txnr: u32, cmd: RelpCommand, datalen: usize, data: String) -> RelpFrame{
        RelpFrame {txnr, cmd, datalen: datalen, data: data}
    }
    
    pub fn from(&mut self, txnr: u32, cmd: RelpCommand, datalen: usize, data: String) {
        self.txnr = txnr;
        self.cmd = cmd;
        self.datalen = datalen;
        self.data = data;
    }

    pub fn from_vec(buf: &Vec<u8>,frame: &mut RelpFrame) -> RelpParserResult {
        get_frame(buf,frame)
    }

    fn to_response(&self, status: RelpStatus, data: &String) -> String {
        let additional_data = match data.is_empty() {
            true => format!("{}", status.to_string()),
            false => format!("{} {}", status.to_string(), data)
        };
        format!("{} {} {} {}\n", self.txnr, RelpCommand::RSP.to_string(), additional_data.len(), &additional_data)
    }
    pub fn ack(&self, data: &String) -> String {
        self.to_response(RelpStatus::OK, data)
    }

    pub fn nack(&self, data: &String) -> String {
        self.to_response(RelpStatus::ERR, data)
    }
}

#[cfg(test)]
mod tests{
}