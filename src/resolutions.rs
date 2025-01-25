/// this many rows and columns are there in a standard SRTM1 file
const EXTENT: usize = 3600;

/// the available resulutions of the SRTM data, in arc seconds
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug, Default)]
pub enum Resolution {
    SRTM05,
    #[default]
    SRTM1,
    SRTM3,
}

impl Resolution {
    /// the number of rows and columns in an SRTM data file of [`Resolution`]
    pub const fn extent(&self) -> usize {
        1 + match self {
            Resolution::SRTM05 => EXTENT * 2,
            Resolution::SRTM1 => EXTENT,
            Resolution::SRTM3 => EXTENT / 3,
        }
    }
    /// total file length in BigEndian, total file length in bytes is [`Resolution::total_len()`] * 2
    pub const fn total_len(&self) -> usize {
        self.extent().pow(2)
    }
}

impl TryFrom<u64> for Resolution {
    type Error = ();

    fn try_from(len: u64) -> Result<Self, Self::Error> {
        let len = usize::try_from(len).map_err(|_| ())?;
        if len == Resolution::SRTM05.total_len() * 2 {
            Ok(Resolution::SRTM05)
        } else if len == Resolution::SRTM1.total_len() * 2 {
            Ok(Resolution::SRTM1)
        } else if len == Resolution::SRTM3.total_len() * 2 {
            Ok(Resolution::SRTM3)
        } else {
            eprintln!("unknown filesize: {len}");
            Err(())
        }
    }
}
