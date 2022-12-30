
use std::str::FromStr;

use crate::{common::{RelpError,RelpCommand, RelpParserResult}, frame::RelpFrame};
const TXNR_MAXLEN: usize = 9;
const CMD_MAXLEN: usize = 32;
const DATALEN_MAXLEN: usize = 9;

pub fn get_frame(buf: &Vec<u8>, dest: &mut RelpFrame) -> RelpParserResult {
    let log = String::from_utf8(buf.to_vec()).unwrap();
    let (cur,_) = match log.find('\n') {
        Some(n) => {
            if n < RelpFrame::MINIMUM_SIZE { // minimum length (e.g:"1 rsp 0")
                return Err(RelpError::InvalidData)
            }
            log.split_at(n)
        },
        None => return Ok(None) // need more data
    };
    let header: Vec<&str> = cur.splitn(4,|n| n == '\n' || n == ' ').collect();

    if header.len() < 3 {
        return Err(RelpError::NeedMoreData)
    }
    let txnr = header[0];
    let cmd = header[1];
    let datalen = header[2];

    let mut header_bytes = 2; // space * 2

    // check txnr
    if txnr.len() > TXNR_MAXLEN {
        return Err(RelpError::InvalidTxnrLength)
    }
    header_bytes += txnr.len();
    let txnr: u32 = match txnr.parse() {
        Ok(n) => n,
        Err(_) => return Err(RelpError::TxnrParseError)
    };
    
    // check cmd
    if cmd.len() > CMD_MAXLEN {
        return Err(RelpError::InvalidCommandLength)
    }
    header_bytes += cmd.len();

    // check datalen
    if datalen.len() > DATALEN_MAXLEN {
        return Err(RelpError::InvalidDataLenLength)
    }
    header_bytes += datalen.len();
    
    let datalen: usize = match datalen.parse() {
        Ok(n) => n,
        Err(_) => return Err(RelpError::DataLenParseError)
    };


    if datalen + header_bytes > log.len() {
        return Ok(None) // need more data
    }

    // if data is available, add size of delimiter
    if datalen > 0 {
        header_bytes += 1; // space
    }

    // calc frame bytes
    let total_bytes = header_bytes + datalen + 1; // add trailer

    // check want bytes
    if total_bytes > buf.len() {
        return Ok(None); // need more data
    }
    let frame = buf.split_at(total_bytes-1).0.to_vec(); // -1 == trailer
    let data = frame.split_at(header_bytes).1; // split to header and data
    let cmd = match RelpCommand::from_str(cmd) {
        Ok(n) => n,
        Err(_) => return Err(RelpError::InvalidCommand),
    };
    dest.from(txnr,cmd,datalen,String::from_utf8(data.to_vec()).unwrap());
    Ok(Some(total_bytes))
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{common::*, frame::RelpFrame};

    const SINGLE_LINE: &str = "1 open 4 test\n";
    const MULTI_LINE: &str = "1 open 9 test\ntest\n";
    const INVALID_TXNR: &str = "a open 0 a\n";
    const INVALID_DATALEN: &str = "1 open a b\n";
    const NO_ADDITIONAL_DATA: &str = "1 open 0\n";
    const TOO_LONG_COMMAND_LENGTH: &str = "1 opennnnnnnnnnnnnnnnnnnnnnnnnnnnnn 0\n";
    const MULTI_FRAME: &str = "1 open 0\n2 close 0\n";
    const NEED_MORE_DATA1: &str = "1 open ";
    const NEED_MORE_DATA2: &str = "1 open 0\n2 open 1";
    const INVALID_COMMAND: &str = "1 abc 0\n2 open 3 abc\n";
    #[test]
    fn test_parse_header_single_line() {
        let ok = RelpFrame::new(
            1,
            RelpCommand::from_str("open").unwrap(),
            4,
            String::from("test"));
        let empty_vec: Vec<u8> = Vec::new();
        let mut buf = Vec::from(SINGLE_LINE);
        let mut frame = RelpFrame::default();
        let nreaded =  get_frame(&buf,&mut frame).unwrap().unwrap();
        buf.drain(0..nreaded);
        assert_eq!(frame,ok);
        assert_eq!(empty_vec,buf);
    }
    #[test]
    fn test_parse_header_multi_line() {
        let ok = RelpFrame::new(
            1,
            RelpCommand::OPEN,
            9,
            String::from("test\ntest"));
        let empty_vec: Vec<u8> = Vec::new();
        let mut buf = Vec::from(MULTI_LINE);
        let mut frame = RelpFrame::default();
        let nreaded =  get_frame(&buf,&mut frame).unwrap().unwrap();
        buf.drain(0..nreaded);
        assert_eq!(frame,ok);
        assert_eq!(empty_vec,buf);
    }
    #[test]
    fn test_parse_header_txnr_parse_err() {
        let mut buf = Vec::from(INVALID_TXNR);
        let mut frame = RelpFrame::default();
        let result = get_frame(&buf, &mut frame);
        
        match result {
            Ok(_n) => assert!(false),
            Err(e) => assert_eq!(e,RelpError::TxnrParseError)
        };
    }
    #[test]
    fn test_parse_header_panic_datalen() {
        let buf = Vec::from(INVALID_DATALEN);
        let mut frame = RelpFrame::default();
        let result = get_frame(&buf, &mut frame);
        match result {
            Ok(_n) => assert!(false),
            Err(e) => assert_eq!(e,RelpError::DataLenParseError)
        };
    }
    #[test]
    fn test_parse_header_panic_cmd_too_long() {
        let buf = Vec::from(TOO_LONG_COMMAND_LENGTH);
        let mut frame = RelpFrame::default();
        let result = get_frame(&buf,&mut frame);
        match result {
            Ok(_n) => assert!(false),
            Err(e) => assert_eq!(e,RelpError::InvalidCommandLength)
        };
    }
    #[test]
    fn test_parse_header_nodata() {
        let buf = Vec::from(NO_ADDITIONAL_DATA);
        let mut frame = RelpFrame::default();
        let nreaded = get_frame(&buf, &mut frame).unwrap().unwrap();
        let ok = RelpFrame::new(
            1,
            RelpCommand::OPEN,
            0,
            String::from(""),
        );
        assert_eq!(frame,ok);

    }

    #[test]
    fn test_multi_frame() {
        let mut buf = Vec::from(MULTI_FRAME);
        let mut first = RelpFrame::default();
        let mut second = RelpFrame::default();
        let mut nreaded = get_frame(&mut buf,&mut first).unwrap().unwrap();
        buf.drain(0..nreaded);        
        nreaded = get_frame(&mut buf,&mut second).unwrap().unwrap();
        let first_ok = RelpFrame::new(
            1,
            RelpCommand::OPEN,
            0,
            String::from("")
        );

        let second_ok = RelpFrame::new(
            2,
            RelpCommand::CLOSE,
            0,
            String::from(""),
        );
        assert_eq!(first,first_ok);
        assert_eq!(second,second_ok);
    }
    #[test]
    fn test_need_more_data() {
        let mut buf = Vec::from(NEED_MORE_DATA1);
        let mut frame = RelpFrame::default();
        match get_frame(&buf,&mut frame).unwrap() {
            Some(_) => assert!(false),
            None => assert!(true)
        };
    

        buf = Vec::from(NEED_MORE_DATA2);
        let mut nreaded = get_frame(& buf, &mut frame).unwrap().unwrap();
        buf.drain(0..nreaded);
        match get_frame(&buf,&mut frame).unwrap() {
            Some(_) => assert!(false),
            None => assert!(true)
        };
    }
    #[test]
    fn test_invalid_command() {
        let buf = Vec::from(INVALID_COMMAND);
        let mut frame = RelpFrame::default();
        match get_frame(&buf,&mut frame) {
            Ok(_) => assert!(false),
            Err(n) => assert_eq!(n, RelpError::InvalidCommand),
        };
        
    }
}
