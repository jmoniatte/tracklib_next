use super::crc::CRC;
use super::presence_column::PresenceColumnView;
use crate::error::Result;
use crate::error::TracklibError;
use nom::{multi::length_data, number::complete::le_u8};
use nom_leb128::{leb128_i64, leb128_u64};

pub(crate) trait Decoder {
    type T;
    fn decode(&mut self) -> Result<Option<Self::T>>;
}

fn validate_column(data: &[u8]) -> Result<&[u8]> {
    const CRC_BYTES: usize = 4;
    let (column_data, crc_bytes) = data.split_at(data.len() - CRC_BYTES);
    let (_, checksum) = CRC::<u32>::parser(column_data)(crc_bytes)?;

    match checksum {
        CRC::Valid(_) => Ok(column_data),
        CRC::Invalid { expected, computed } => {
            Err(TracklibError::CRC32Error { expected, computed })
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct I64Decoder<'a> {
    data: &'a [u8],
    presence_column_view: PresenceColumnView<'a>,
    prev: i64,
}

impl<'a> I64Decoder<'a> {
    pub(crate) fn new(
        data: &'a [u8],
        presence_column_view: PresenceColumnView<'a>,
    ) -> Result<Self> {
        let column_data = validate_column(data)?;

        Ok(Self {
            data: column_data,
            presence_column_view,
            prev: 0,
        })
    }
}

impl<'a> Decoder for I64Decoder<'a> {
    type T = i64;

    fn decode(&mut self) -> Result<Option<Self::T>> {
        if let Some(is_present) = self.presence_column_view.next() {
            if is_present {
                let (rest, value) = leb128_i64(self.data)?;
                let new = self.prev + value;
                self.prev = new;
                self.data = rest;
                Ok(Some(new))
            } else {
                Ok(None)
            }
        } else {
            // ran out of rows, this is an error
            return Err(TracklibError::ParseIncompleteError {
                needed: nom::Needed::Unknown,
            });
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct BoolDecoder<'a> {
    data: &'a [u8],
    presence_column_view: PresenceColumnView<'a>,
}

impl<'a> BoolDecoder<'a> {
    pub(crate) fn new(
        data: &'a [u8],
        presence_column_view: PresenceColumnView<'a>,
    ) -> Result<Self> {
        let column_data = validate_column(data)?;

        Ok(Self {
            data: column_data,
            presence_column_view,
        })
    }
}

impl<'a> Decoder for BoolDecoder<'a> {
    type T = bool;

    fn decode(&mut self) -> Result<Option<Self::T>> {
        if let Some(is_present) = self.presence_column_view.next() {
            if is_present {
                let (rest, value) = le_u8(self.data)?;
                self.data = rest;
                Ok(Some(value != 0))
            } else {
                Ok(None)
            }
        } else {
            // ran out of rows, this is an error
            return Err(TracklibError::ParseIncompleteError {
                needed: nom::Needed::Unknown,
            });
        }
    }
}

#[cfg_attr(test, derive(Debug))]
pub(crate) struct StringDecoder<'a> {
    data: &'a [u8],
    presence_column_view: PresenceColumnView<'a>,
}

impl<'a> StringDecoder<'a> {
    pub(crate) fn new(
        data: &'a [u8],
        presence_column_view: PresenceColumnView<'a>,
    ) -> Result<Self> {
        let column_data = validate_column(data)?;

        Ok(Self {
            data: column_data,
            presence_column_view,
        })
    }
}

impl<'a> Decoder for StringDecoder<'a> {
    type T = String;

    fn decode(&mut self) -> Result<Option<Self::T>> {
        if let Some(is_present) = self.presence_column_view.next() {
            if is_present {
                let (rest, string_bytes) = length_data(leb128_u64)(self.data)?;
                self.data = rest;
                Ok(Some(String::from_utf8(string_bytes.to_vec()).unwrap()))
            } else {
                Ok(None)
            }
        } else {
            // ran out of rows, this is an error
            return Err(TracklibError::ParseIncompleteError {
                needed: nom::Needed::Unknown,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::read::presence_column::parse_presence_column;
    use assert_matches::assert_matches;

    #[test]
    fn test_decode_i64() {
        #[rustfmt::skip]
        let presence_buf = &[0b00000001,
                             0b00000001,
                             0b00000000,
                             0b00000001,
                             0b00000001,
                             0x32, // crc
                             0x65,
                             0x57,
                             0xFB];
        assert_matches!(parse_presence_column(1, 5)(presence_buf), Ok((&[], presence_column)) => {
            let presence_column_view = presence_column.view(0);
            #[rustfmt::skip]
            let buf = &[0x20,
                        0x42,
                        0x00,
                        0x1,
                        0x0C, // crc
                        0x01,
                        0x49,
                        0xA3];
            assert_matches!(I64Decoder::new(buf, presence_column_view), Ok(mut decoder) => {
                assert_matches!(decoder.decode(), Ok(Some(32)));
                assert_matches!(decoder.decode(), Ok(Some(-30)));
                assert_matches!(decoder.decode(), Ok(None));
                assert_matches!(decoder.decode(), Ok(Some(-30)));
                assert_matches!(decoder.decode(), Ok(Some(-29)));
            });
        });
    }

    #[test]
    fn test_decode_bool() {
        #[rustfmt::skip]
        let presence_buf = &[0b00000000,
                             0b00000001,
                             0b00000001,
                             0x94, // crc
                             0x5E,
                             0x43,
                             0x9E];
        assert_matches!(parse_presence_column(1, 3)(presence_buf), Ok((&[], presence_column)) => {
            let presence_column_view = presence_column.view(0);
            #[rustfmt::skip]
            let buf = &[0x01,
                        0x00,
                        0x5E, // crc
                        0x5A,
                        0x51,
                        0x2D];
            assert_matches!(BoolDecoder::new(buf, presence_column_view), Ok(mut decoder) => {
                assert_matches!(decoder.decode(), Ok(None));
                assert_matches!(decoder.decode(), Ok(Some(true)));
                assert_matches!(decoder.decode(), Ok(Some(false)));
            });
        });
    }

    #[test]
    fn test_decode_string() {
        #[rustfmt::skip]
        let presence_buf = &[0b00000000,
                             0b00000001,
                             0b00000001,
                             0x94, // crc
                             0x5E,
                             0x43,
                             0x9E];
        assert_matches!(parse_presence_column(1, 3)(presence_buf), Ok((&[], presence_column)) => {
            let presence_column_view = presence_column.view(0);
            #[rustfmt::skip]
            let buf = &[0x01,
                        b'R',
                        0x03,
                        b'i',
                        b'd',
                        b'e',
                        0x73, // crc
                        0x91,
                        0x5A,
                        0x74];
            assert_matches!(StringDecoder::new(buf, presence_column_view), Ok(mut decoder) => {
                assert_matches!(decoder.decode(), Ok(None));
                assert_matches!(decoder.decode(), Ok(Some(s)) => {
                    assert_eq!(s, "R");
                });
                assert_matches!(decoder.decode(), Ok(Some(s)) => {
                    assert_eq!(s, "ide");
                });
            });
        });
    }

    #[test]
    fn test_decode_bad_crc() {
        #[rustfmt::skip]
        let presence_buf = &[0b00000000,
                             0b00000001,
                             0b00000001,
                             0x94, // crc
                             0x5E,
                             0x43,
                             0x9E];
        assert_matches!(parse_presence_column(1, 3)(presence_buf), Ok((&[], presence_column)) => {
            let presence_column_view = presence_column.view(0);
            #[rustfmt::skip]
            let buf = &[0x00,
                        0x01,
                        0x02,
                        0x00, // invalid crc
                        0x00,
                        0x00,
                        0x00];
            assert_matches!(StringDecoder::new(buf, presence_column_view),
                            Err(crate::error::TracklibError::CRC32Error{expected: 0x00,
                                                                        computed: 0x9300784D}));
        });
    }
}
