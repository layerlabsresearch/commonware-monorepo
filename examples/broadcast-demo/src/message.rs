use bytes::{Buf, BufMut};
use commonware_codec::{EncodeSize, Error as CodecError, RangeCfg, Read, ReadRangeExt, ReadExt, Write};
use commonware_cryptography::{hash, sha256::Digest as Sha256Digest, Committable, Digestible};

#[derive(Debug, Clone)]
pub struct Message {
    pub id: u32,
    pub data: Vec<u8>,
}

impl Message {
    pub fn new(id: u32, data: impl Into<Vec<u8>>) -> Self {
        Self { id, data: data.into() }
    }

    pub fn commitment_for_id(id: u32) -> Sha256Digest {
        hash(&id.to_le_bytes())
    }
}

impl Digestible<Sha256Digest> for Message {
    fn digest(&self) -> Sha256Digest {
        hash(&self.data)
    }
}

impl Committable<Sha256Digest> for Message {
    fn commitment(&self) -> Sha256Digest {
        Self::commitment_for_id(self.id)
    }
}

impl Write for Message {
    fn write(&self, buf: &mut impl BufMut) {
        self.id.write(buf);
        self.data.write(buf);
    }
}

impl EncodeSize for Message {
    fn encode_size(&self) -> usize {
        self.id.encode_size() + self.data.encode_size()
    }
}

impl Read for Message {
    type Cfg = RangeCfg;

    fn read_cfg(buf: &mut impl Buf, range: &Self::Cfg) -> Result<Self, CodecError> {
        let id = u32::read(buf)?;
        let data = Vec::<u8>::read_range(buf, *range)?;
        Ok(Self { id, data })
    }
}
