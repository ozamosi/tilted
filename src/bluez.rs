use libc::{c_ushort, c_void};
use std::{
    io::{self, Write},
    os::unix::{
        io::{AsRawFd, RawFd},
        net::UnixStream,
    },
};

#[repr(packed(4))]
#[derive(Debug, Copy, Clone, Default)]
pub struct HciFilter {
    type_mask: u32,
    event_mask: u64,
    opcode: u16,
}

impl HciFilter {
    pub fn new(type_mask: HciType, event_mask: HciEvent) -> HciFilter {
        HciFilter {
            type_mask: 1 << (type_mask as u32),
            event_mask: event_mask as u64,
            opcode: 0,
        }
    }
}

#[repr(u64)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum HciEvent {
    InquiryComplete = 1 << 0x01,
    InquiryResult = 1 << 0x02,
    ConnComplete = 1 << 0x03,
    ConnRequest = 1 << 0x04,
    DisconnComplete = 1 << 0x05,
    AuthComplete = 1 << 0x06,
    RemoteNameReqComplete = 1 << 0x07,
    EncryptChange = 1 << 0x08,
    ChangeConnLinkKeyComplete = 1 << 0x09,
    MasterLinkKeyComplete = 1 << 0x0A,
    ReadRemoteFeaturesComplete = 1 << 0x0B,
    ReadRemoteVersionComplete = 1 << 0x0C,
    QosSetupComplete = 1 << 0x0D,
    CmdComplete = 1 << 0x0E,
    CmdStatus = 1 << 0x0F,
    HardwareError = 1 << 0x10,
    RoleChange = 1 << 0x12,
    NumCompPkts = 1 << 0x13,
    ModeChange = 1 << 0x14,
    ReturnLinkKeys = 1 << 0x15,
    PinCodeReq = 1 << 0x16,
    LinkKeyReq = 1 << 0x17,
    LinkKeyNotify = 1 << 0x18,
    LoopbackCommand = 1 << 0x19,
    DataBufferOverflow = 1 << 0x1A,
    MaxSlotsChange = 1 << 0x1B,
    ReadClockOffsetComplete = 1 << 0x1C,
    ConnPtypeChanged = 1 << 0x1D,
    QosViolation = 1 << 0x1E,
    PscanRepModeChange = 1 << 0x20,
    FlowSpecComplete = 1 << 0x21,
    InquiryResultWithRssi = 1 << 0x22,
    ReadRemoteExtFeaturesComplete = 1 << 0x23,
    SyncConnComplete = 1 << 0x2C,
    SyncConnChanged = 1 << 0x2D,
    SniffSubrating = 1 << 0x2E,
    ExtendedInquiryResult = 1 << 0x2F,
    EncryptionKeyRefreshComplete = 1 << 0x30,
    IoCapabilityRequest = 1 << 0x31,
    IoCapabilityResponse = 1 << 0x32,
    UserConfirmRequest = 1 << 0x33,
    UserPasskeyRequest = 1 << 0x34,
    RemoteOobDataRequest = 1 << 0x35,
    SimplePairingComplete = 1 << 0x36,
    LinkSupervisionTimeoutChanged = 1 << 0x38,
    EnhancedFlushComplete = 1 << 0x39,
    UserPasskeyNotify = 1 << 0x3B,
    KeypressNotify = 1 << 0x3C,
    RemoteHostFeaturesNotify = 1 << 0x3D,
    LeMetaEvent = 1 << 0x3E,
}

#[repr(u16)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
enum BtProto {
    L2CAP = 0,
    HCI = 1,
    RFCOMM = 3,
    AVDTP = 7,
}

#[repr(i32)]
#[allow(dead_code)]
enum Sol {
    HCI = 0,
    L2CAP = 6,
    SCO = 17,
    RFCOMM = 18,
    BLUETOOTH = 274,
}

#[repr(u16)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
enum HciChannel {
    Raw = 0,
    User = 1,
    Monitor = 2,
    Control = 3,
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
pub enum HciType {
    CommandPkt = 1,
    AclDataPkt = 2,
    ScoDataPkt = 3,
    EventPkt = 4,
    VendorPkt = 0xff,
}

impl From<HciType> for [u8; 4] {
    fn from(hcitype: HciType) -> [u8; 4] {
        (hcitype as u32).to_le_bytes()
    }
}

#[repr(i32)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
enum HciSocketOption {
    DataDir = 1,
    Filter = 2,
    TimeStamp = 3,
}

const HCI_DEV_NONE: c_ushort = 0;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct SockAddrHci {
    pub hci_family: c_ushort,
    pub hci_dev: c_ushort,
    pub hci_channel: HciChannel,
}

// TODO: Ogf and Ocf should be the same command type
#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
pub enum Ogf {
    None = 0,
    LinkControl = 0x01,
    LinkPolicy = 0x02,
    HostCtl = 0x03,
    InfoParam = 0x04,
    StatusParam = 0x05,
    TestingCmd = 0x3e,
    LeCtl = 0x08,
    VendorCmd = 0x3f,
}

pub enum Ocf {
    LeCtl(LeCtl),
}

impl From<Ocf> for u16 {
    fn from(ocf: Ocf) -> u16 {
        match ocf {
            Ocf::LeCtl(lectl) => lectl as u16,
        }
    }
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
pub enum LeCtl {
    SetEventMask = 0x01,
    ReadBufferSize = 0x02,
    ReadLocalSupportedFeatures = 0x03,
    SetRandomAddress = 0x05,
    SetAdvertisingParameters = 0x06,
    ReadAdvertisingChanelTxPower = 0x07,
    SetAdvertisingData = 0x08,
    SetScanResponseData = 0x09,
    SetAdvertiseEnable = 0x0a,
    SetScanParameters = 0x0b,
    SetScanEnable = 0x0c,
    CreateConn = 0x0d,
    CreateConnCancel = 0x0e,
    ReadWhiteListSize = 0x0f,
    ClearWhiteList = 0x10,
    AddDeviceToWhiteList = 0x11,
    RemoveDeviceFromWhiteList = 0x12,
    ConnUpdate = 0x013,
    SetHostChannelClassification = 0x14,
    ReadChannelMap = 0x15,
    ReadRemoteUsedFeatures = 0x16,
    Encrypt = 0x17,
    Rand = 0x18,
    StartEncryption = 0x19,
    LtkReply = 0x1a,
    LtkNegReply = 0x1b,
    ReadSupportedStates = 0x1c,
    ReceiverList = 0x1d,
    TransmitterTest = 0x1e,
    TestEnd = 0x1f,
    AddDeviceToResolvList = 0x27,
    RemoveDeviceFromResolvList = 0x28,
    ClearResolvList = 0x29,
    ReadResolvListSize = 0x2a,
    SetAddressResolutionEnable = 0x2d,
}

#[repr(packed)]
#[allow(dead_code)]
struct HciCommandHdr {
    opcode: Opcode,
    plen: u8,
}

struct Opcode(u16);

impl From<(Ogf, Ocf)> for Opcode {
    fn from((ogf, ocf): (Ogf, Ocf)) -> Opcode {
        let ogf = ogf as u16;
        let ocf: u16 = ocf.into();
        Opcode((ogf << 10) & (ocf & 0x3ff))
    }
}

impl From<Opcode> for [u8; 2] {
    fn from(opcode: Opcode) -> [u8; 2] {
        opcode.0.to_le_bytes()
    }
}

pub fn open() -> Result<RawFd, io::Error> {
    let fd: RawFd = unsafe {
        libc::socket(
            libc::AF_BLUETOOTH,
            libc::SOCK_RAW | libc::SOCK_CLOEXEC,
            BtProto::HCI as libc::c_int,
        )
    };

    if fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let addr = SockAddrHci {
        hci_family: libc::AF_BLUETOOTH as u16,
        hci_dev: HCI_DEV_NONE,
        hci_channel: HciChannel::Raw,
    };

    if unsafe {
        libc::bind(
            fd,
            &addr as *const SockAddrHci as *const libc::sockaddr,
            std::mem::size_of::<SockAddrHci>() as u32,
        )
    } < 0
    {
        let err = io::Error::last_os_error();

        unsafe {
            libc::close(fd);
        }

        return Err(err);
    }

    Ok(fd)
}

// TODO: this bit flipping should be on the type level
pub fn enable_le_scan(stream: &mut UnixStream) -> Result<(), io::Error> {
    let ogf = Ogf::LeCtl;
    let ocf = Ocf::LeCtl(LeCtl::SetScanEnable);
    let opcode: Opcode = (ogf, ocf).into();
    let opcode: [u8; 2] = opcode.into();
    let hci_type: [u8; 4] = HciType::CommandPkt.into();
    let mut buf = [0u8; 4 + 2 + 3];
    buf[0..4].copy_from_slice(&hci_type);
    buf[4..6].copy_from_slice(&opcode);
    buf[6] = 2; // len
    buf[7] = 1; // enable?
    buf[8] = 1; // repeat?
    stream.write_all(&buf)?;
    Ok(())
}

pub fn get_filter(stream: &UnixStream) -> Result<HciFilter, io::Error> {
    let filter = HciFilter::default();

    if unsafe {
        libc::setsockopt(
            stream.as_raw_fd(),
            Sol::HCI as i32,
            HciSocketOption::Filter as i32,
            &filter as *const HciFilter as *const c_void,
            std::mem::size_of::<HciFilter>() as u32,
        )
    } < 0
    {
        let err = io::Error::last_os_error();
        return Err(err);
    }
    Ok(filter)
}

pub fn set_filter(stream: &UnixStream, filter: HciFilter) -> Result<(), io::Error> {
    if unsafe {
        libc::setsockopt(
            stream.as_raw_fd(),
            Sol::HCI as i32,
            HciSocketOption::Filter as i32,
            &filter as *const HciFilter as *const c_void,
            std::mem::size_of::<HciFilter>() as u32,
        )
    } < 0
    {
        let err = io::Error::last_os_error();
        return Err(err);
    }
    Ok(())
}
