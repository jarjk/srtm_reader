/// the available resolutions of the SRTM data, in arc seconds
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug, Default)]
pub enum Resolution {
    SRTM05,
    #[default]
    SRTM1,
    SRTM3,
}

impl Resolution {
    /// this many rows and columns are there in a standard SRTM1 file
    const DEFAULT_EXTENT: usize = 3600;

    /// a record of elevation takes up this many bytes
    pub(crate) const BYTES_PER_ELEVATION: usize = 2;

    /// the number of rows and columns in an SRTM data file of [`Resolution`]
    pub const fn extent(&self) -> usize {
        1 + match self {
            Resolution::SRTM05 => Self::DEFAULT_EXTENT * 2,
            Resolution::SRTM1 => Self::DEFAULT_EXTENT,
            Resolution::SRTM3 => Self::DEFAULT_EXTENT / 3,
        }
    }
    /// total file length in BigEndian, total file length in bytes is [`Resolution::total_len()`] * [`Resolution::BYTES_PER_ELEVATION`]
    pub const fn total_len(&self) -> usize {
        self.extent().pow(2)
    }
}

impl TryFrom<u64> for Resolution {
    type Error = std::io::Error;

    fn try_from(len: u64) -> Result<Self, Self::Error> {
        let len = usize::try_from(len).map_err(|e| crate::new_err!(of: e))?;
        if len == Resolution::SRTM05.total_len() * Self::BYTES_PER_ELEVATION {
            Ok(Resolution::SRTM05)
        } else if len == Resolution::SRTM1.total_len() * Self::BYTES_PER_ELEVATION {
            Ok(Resolution::SRTM1)
        } else if len == Resolution::SRTM3.total_len() * Self::BYTES_PER_ELEVATION {
            Ok(Resolution::SRTM3)
        } else {
            let error = crate::new_err!(Unsupported, format!("unsupported filesize: {len}"));
            Err(error)
        }
    }
}
