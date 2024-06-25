use crate::{LogLevel, LogType};

pub const SOCKET_FILE_DIR: &str = "/dev/unix/socket/";
pub const INPUT_SOCKET_NAME: &str = "hilogInput";

pub const HILOG_SOCKET_PATH: &str = "/dev/unix/socket/hilogInput";

/// maximum length of a log, including '\0'
pub const MAX_LOG_LEN: usize = 4096;
/// log tag size, include '\0'
pub const MAX_TAG_LEN: usize = 32;

pub struct TagLen(u16);

impl TagLen {
    pub const fn new(len: usize) -> Self {
        if len > MAX_TAG_LEN {
            panic!("len exceeded MAX_TAG_LEN");
        }
        TagLen(len as u16)
    }
}

/// Corresponds to the following C bitfield
///
/// ```c
/// uint16_t version : 3;
//  uint16_t type : 4; /* APP,CORE,INIT,SEC etc */
//  uint16_t level : 3;
//  uint16_t tagLen : 6; /* include '\0' */
// ```
#[repr(transparent)]
pub struct MessageMetaField(u16);

impl MessageMetaField {
    const VERSION_BITS: usize = 3;
    const TYPE_BITS: usize = 4;
    const LEVEL_BITS: usize = 3;
    const TAG_LEN_BITS: usize = 6;
    const VERSION_OFFSET: usize = 0;
    const TYPE_OFFSET: usize = Self::VERSION_OFFSET + Self::VERSION_BITS;
    const LEVEL_OFFSET: usize = Self::TYPE_OFFSET + Self::TYPE_BITS;
    const TAG_LEN_OFFSET: usize = Self::LEVEL_OFFSET + Self::LEVEL_BITS;
    // const TOTAL_OFFSET: usize = Self::VERSION_OFFSET + Self::VERSION_BITS;
    // Note: May change in future versions of OH / hilog.
    const VERSION: u16 = 0;

    pub fn new(log_type: LogType, level: LogLevel, tag_len: TagLen) -> Self {
        debug_assert!(tag_len.0 < (1 << Self::TAG_LEN_BITS));
        Self(
            Self::VERSION << Self::VERSION_OFFSET
                | (log_type.0 as u16) << Self::TYPE_OFFSET
                | (level.0 as u16) << Self::VERSION_OFFSET
                | (tag_len.0 as u16) << Self::TAG_LEN_OFFSET,
        )
    }
}

#[repr(C, packed)]
pub struct HilogMsg {
    pub len: u16,
    pub meta_bitfield: MessageMetaField,
    pub tv_sec: u32,
    pub tv_nsec: u32,
    pub mono_sec: u32,
    pub pid: u32,
    pub tid: u32,
    pub domain: u32,
}

impl HilogMsg {
    pub const fn as_bytes(&self) -> &[u8] {
        // SAFETY: Upholds all preconditions trivially, since we
        // simpy reborrow self as bytes, so alignment, lifetimes
        // and validity are not an issue.
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}
