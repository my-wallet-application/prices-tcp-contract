use my_tcp_sockets::TcpWriteBuffer;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub const MESSAGE_SPLITTER: &[u8; 1] = b" ";

#[derive(Debug, Clone)]
pub struct BidAskDataTcpModel {
    pub instrument_id: String,
    pub bid: f64,
    pub ask: f64,
    pub volume: f64,
    pub timestamp: DateTimeAsMicroseconds,
}

impl BidAskDataTcpModel {
    pub fn serialize(&self, out: &mut impl TcpWriteBuffer) {
        out.write_byte(b'A');
        out.write_slice(MESSAGE_SPLITTER);
        out.write_slice(self.instrument_id.as_bytes());
        out.write_slice(MESSAGE_SPLITTER);
        out.write_byte(b'B');
        out.write_slice(format!("{}", self.bid).as_bytes());
        out.write_slice(MESSAGE_SPLITTER);
        out.write_byte(b'A');
        out.write_slice(format!("{}", self.ask).as_bytes());
        out.write_slice(MESSAGE_SPLITTER);
        out.write_slice(format!("{}", self.volume).as_bytes());
        out.write_slice(MESSAGE_SPLITTER);
        write_date(out, self.timestamp);
    }

    pub fn deserialize(src: &[u8]) -> Result<Self, SerializeError> {
        let mut no = 0;
        let mut instrument_id = None;
        let mut bid = None;
        let mut ask = None;
        let mut volume = None;
        let mut date_time = None;

        for itm in src.split(|x| *x == b' ') {
            match no {
                1 => instrument_id = std::str::from_utf8(itm).unwrap().into(),
                2 => bid = std::str::from_utf8(itm).unwrap().into(),
                3 => ask = std::str::from_utf8(itm).unwrap().into(),
                4 => volume = std::str::from_utf8(itm).unwrap().into(),
                5 => date_time = itm.into(),
                _ => {}
            }

            no += 1;
        }

        let instrument_id = instrument_id.unwrap();
        let bid = bid.unwrap();
        let bid = if bid.starts_with("B") {
            bid[1..].parse().unwrap()
        } else {
            bid.parse().unwrap()
        };

        let ask = ask.unwrap();

        let ask = if ask.starts_with("A") {
            ask[1..].parse().unwrap()
        } else {
            ask.parse().unwrap()
        };

        let volume = volume.unwrap();
        let volume = volume.parse().unwrap();

        let date_time = date_time.unwrap();
        let timestamp = crate::date_utils::parse_tcp_feed_date(&date_time);

        Ok(Self {
            instrument_id: instrument_id.to_string(),
            bid,
            ask,
            volume,
            timestamp,
        })
    }
}

#[derive(Debug)]
pub enum SerializeError {
    InvalidDate,
    InvalidDateMarker,
    MissingDateMarker,
    DateSerializeError,
}

fn write_date(out: &mut impl TcpWriteBuffer, dt: DateTimeAsMicroseconds) {
    let str = dt.to_rfc3339();
    let str = str.as_bytes();
    out.write_slice(&str[0..4]);
    out.write_slice(&str[5..7]);
    out.write_slice(&str[8..10]);
    out.write_slice(&str[11..13]);
    out.write_slice(&str[14..16]);
    if str[19] == b'+' {
        out.write_slice(&str[17..19]);
    } else {
        out.write_slice(&str[17..23]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let message = b"A EURUSD B1.55555 A2.55555 50000000 20230213142225.555";
        let result = BidAskDataTcpModel::deserialize(message).unwrap();
        assert_eq!(result.instrument_id, "EURUSD");
        assert_eq!(result.bid, 1.55555);
        assert_eq!(result.ask, 2.55555);
        assert_eq!(result.volume, 50000000.0);
    }

    #[test]
    fn test_serialize() {
        let message = "A EURUSD B1.55555 A2.55555 50000000 20230213142225.555";

        let result = BidAskDataTcpModel {
            instrument_id: "EURUSD".to_string(),
            bid: 1.55555,
            ask: 2.55555,
            volume: 50000000.0,
            timestamp: DateTimeAsMicroseconds::from_str("2023-02-13T14:22:25.555").unwrap(),
        };

        let mut serialized: Vec<u8> = Vec::new();

        result.serialize(&mut serialized);

        assert_eq!(String::from_utf8(serialized).unwrap(), message);
    }

    #[test]
    fn test_write_date() {
        let dt = DateTimeAsMicroseconds::from_str("2015-05-12T12:13:14.1").unwrap();

        let mut buffer = Vec::new();

        write_date(&mut buffer, dt);

        println!("{:?}", std::str::from_utf8(buffer.as_slice()).unwrap());
    }
}
