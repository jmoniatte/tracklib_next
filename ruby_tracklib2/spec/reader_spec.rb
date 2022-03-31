require "spec_helper"

data = [
  # Header
  0x89, # rwtfmagic
  0x52,
  0x57,
  0x54,
  0x46,
  0x0A,
  0x1A,
  0x0A,
  0x01, # file version
  0x00, # fv reserve
  0x00,
  0x00,
  0x00, # creator version
  0x00, # cv reserve
  0x00,
  0x00,
  0x18, # metadata table offset
  0x00,
  0x23, # data offset
  0x00,
  0x00, # e reserve
  0x00,
  0x89, # header crc
  0x98,

  # Metadata Table
  0x01, # one entry
  0x00, # entry type: track_type = 0x00
  0x05, # two byte entry size = 5
  0x00,
  0x02, # track type: segment = 0x02
  0x05, # four byte segment ID
  0x00,
  0x00,
  0x00,
  0xD4, # crc
  0x93,

  # Data Table
  0x02, # two sections

  # Data Table Section 1
  0x00, # section encoding = standard
  0x05, # leb128 point count
  0x33, # leb128 data size

  # Schema for Section 1
  0x00, # schema version
  0x03, # field count
  0x00, # first field type = I64
  0x01, # name len
  0x6D, # name = m
  0x09, # leb128 data size
  0x05, # second field type = Bool
  0x01, # name len
  0x6B, # name = k
  0x09, # leb128 data size
  0x04, # third field type = String
  0x01, # name len
  0x6A, # name = j
  0x18, # leb128 data size

  # Data Table Section 2
  0x00, # section encoding = standard
  0x03, # leb128 point count
  0x26, # leb128 data size

  # Schema for Section 2
  0x00, # schema version
  0x03, # field count
  0x00, # first field type = I64
  0x01, # name length
  0x61, # name = a
  0x07, # leb128 data size
  0x05, # second field type = Bool
  0x01, # name length
  0x62, # name = b
  0x06, # leb128 data size
  0x04, # third field type = String
  0x01, # name length
  0x63, # name = c
  0x12, # leb128 data size

  # Data Table CRC
  0xC8,
  0x42,

  # Data Section 1

  # Presence Column
  0b00000111,
  0b00000111,
  0b00000111,
  0b00000111,
  0b00000111,
  0xF6, # crc
  0xF8,
  0x0D,
  0x73,

  # Data Column 1 = I64
  0x2A, # 42
  0x00, # no change
  0x00, # no change
  0x00, # no change
  0x00, # no change
  0xD0, # crc
  0x8D,
  0x79,
  0x68,
  
  # Data Column 2 = Bool
  0x01, # true
  0x01, # true
  0x01, # true
  0x01, # true
  0x01, # true
  0xB5, # crc
  0xC9,
  0x8F,
  0xFA,

  # Data Column 3 = String
  0x03, # length 3
  0x68, # h
  0x65, # e
  0x79, # y
  0x03, # length 3
  0x68, # h
  0x65, # e
  0x79, # y
  0x03, # length 3
  0x68, # h
  0x65, # e
  0x79, # y
  0x03, # length 3
  0x68, # h
  0x65, # e
  0x79, # y
  0x03, # length 3
  0x68, # h
  0x65, # e
  0x79, # y
  0x36, # crc
  0x71,
  0x24,
  0x0B,

  # Data Section 2

  # Presence Column
  0b00000111,
  0b00000101,
  0b00000111,
  0x1A, # crc
  0x75,
  0xEA,
  0xC4,

  # Data Column 1 = I64
  0x01, # 1
  0x01, # 2
  0x02, # 4
  0xCA, # crc
  0xD4,
  0xD8,
  0x92,

  # Data Column 2 = Bool
  0x00, # false
  # None
  0x01, # true
  0x35, # crc
  0x86,
  0x89,
  0xFB,

  # Data Column 3 = String
  0x04, # length 4
  0x52, # R
  0x69, # i
  0x64, # d
  0x65, # e
  0x04, # length 4
  0x77, # w
  0x69, # i
  0x74, # t
  0x68, # h
  0x03, # length 3
  0x47, # G
  0x50, # P
  0x53, # S
  0xA3, # crc
  0x02,
  0xEC,
  0x48
].pack("c*")

describe TrackReader do
  it "can read metadata" do
    track_reader = TrackReader.new(data)
    expect(track_reader.metadata()).to eq([[:track_type, :segment, 5]])
  end

  it "can read versions" do
    track_reader = TrackReader.new(data)
    expect(track_reader.file_version()).to eq(1)
    expect(track_reader.creator_version()).to eq(0)
  end
end
