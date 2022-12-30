use std::str::FromStr;

pub const RELP_PROTOCOL_VERSION: &'static str = "0"; // librelp used "0"
pub const RELP_SUPPORTED_COMMAND: &'static str = "syslog";
pub const RELPRS_SOFTWARE_NAME: &'static str = "relp_for_rust" ;
pub const RELPRS_SOFTWARE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug,PartialEq)]
pub enum RelpError {
    InvalidTxnrLength,
    InvalidCommandLength,
    InvalidDataLenLength,
    TxnrParseError,
    DataLenParseError,
    NeedMoreData,
    InvalidData,
    InvalidCommand,
    StreamReadError,
    AlreadyOpened,
    UnknownError,
}

#[derive(Debug,PartialEq,Default)]
pub enum RelpCommand {
    OPEN,
    SYSLOG,
    CLOSE,
    RSP,
    ABORT,
    STARTTLS,
    #[default]
    UNKNOWN,
}

impl FromStr for RelpCommand {
    fn from_str(cmd: &str) -> Result<RelpCommand,()>{
        match cmd {
            "open" => Ok(RelpCommand::OPEN),
            "syslog" => Ok(RelpCommand::SYSLOG),
            "close" => Ok(RelpCommand::CLOSE),
            "rsp" => Ok(RelpCommand::RSP),
            "abort" => Ok(RelpCommand::ABORT),
            "starttls" => Ok(RelpCommand::STARTTLS),
            _ => Err(()),
        }
    }

    type Err = ();
}
impl ToString for RelpCommand {
    fn to_string(&self) -> String {
        match *self {
            RelpCommand::OPEN => "open".to_string(),
            RelpCommand::SYSLOG => "syslog".to_string(),
            RelpCommand::CLOSE => "close".to_string(),
            RelpCommand::RSP => "rsp".to_string(),
            RelpCommand::ABORT => "abort".to_string(),
            RelpCommand::STARTTLS => "starttls".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum RelpStatus{
    OK,
    ERR,
}

impl FromStr for RelpStatus {
    fn from_str(str: &str) -> Result<RelpStatus,RelpError> {
        match str {
            "200" => Ok(RelpStatus::OK),
            "500" => Ok(RelpStatus::ERR),
            _ => Err(RelpError::InvalidData)
        }
    }

    type Err=RelpError;
}

impl ToString for RelpStatus{
    fn to_string(&self) -> String {
        match self {
            RelpStatus::OK => "200".to_string(),
            RelpStatus::ERR => "500".to_string(),
        }
    }
}
pub type RelpParserResult = Result<Option<usize>, RelpError>;


#[cfg(test)]
mod tests {
    use crate::frame::RelpFrame;
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(RelpCommand::from_str("open").unwrap(),RelpCommand::OPEN);
        assert_eq!(RelpCommand::from_str("close").unwrap(),RelpCommand::CLOSE);
        assert_eq!(RelpCommand::from_str("abort").unwrap(),RelpCommand::ABORT);
        assert_eq!(RelpCommand::from_str("rsp").unwrap(),RelpCommand::RSP);
        assert_eq!(RelpCommand::from_str("starttls").unwrap(),RelpCommand::STARTTLS);
        assert_eq!(RelpCommand::from_str("syslog").unwrap(),RelpCommand::SYSLOG);
        match RelpCommand::from_str("unknown") {
            Ok(_n) => assert!(false),
            Err(_) => assert!(true)
        }
    }

    #[test]
    fn test_to_string() {
        assert_eq!(RelpCommand::OPEN.to_string(),"open");
        assert_eq!(RelpCommand::CLOSE.to_string(),"close");
        assert_eq!(RelpCommand::ABORT.to_string(),"abort");
        assert_eq!(RelpCommand::RSP.to_string(),"rsp");
        assert_eq!(RelpCommand::STARTTLS.to_string(),"starttls");
        assert_eq!(RelpCommand::SYSLOG.to_string(),"syslog");
        assert_eq!(RelpCommand::UNKNOWN.to_string(),"unknown");
    }

    #[test]
    fn test_ack() {
        let ok = RelpFrame::new(1, RelpCommand::OPEN, 0, "".to_string());
        assert_eq!(ok.ack(&"".to_string()), "1 rsp 3 200\n");
        let ok2 = RelpFrame::new(1, RelpCommand::OPEN, 0, "".to_string());
        assert_eq!(ok2.ack(&"OK".to_string()), "1 rsp 6 200 OK\n")
    }

    #[test]
    fn test_nack() {
        let ok = RelpFrame::new(1, RelpCommand::OPEN, 0, "".to_string());
        assert_eq!(ok.nack(&"".to_string()), "1 rsp 3 500\n");
        let ok2 = RelpFrame::new(1, RelpCommand::OPEN, 0, "".to_string());
        assert_eq!(ok2.nack(&"ERR".to_string()), "1 rsp 7 500 ERR\n");
    }
}