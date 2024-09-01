use byteorder::{BigEndian, ByteOrder, LittleEndian};

/// SCSI command codes
#[repr(u8)]
pub enum ScsiCommand {
    TestUnitReady = 0x00,
    RequestSense = 0x03,
    Inquiry = 0x12,
    ModeSense6 = 0x1A,
    StartStopUnit = 0x1B,
    PreventAllowMediumRemoval = 0x1E,
    ReadFormatCapacities = 0x23,
    ReadCapacity = 0x25,
    Read10 = 0x28,
    Write10 = 0x2A,
    Verify10 = 0x2F,
}

/// SCSI Inquiry command structure
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum SenseKey {
    NoSense = 0x00,
    RecoveredError = 0x01,
    NotReady = 0x02,
    MediumError = 0x03,
    HardwareError = 0x04,
    IllegalRequest = 0x05,
    UnitAttention = 0x06,
    DataProtect = 0x07,
    BlankCheck = 0x08,
    VendorSpecific = 0x09,
    CopyAborted = 0x0A,
    AbortedCommand = 0x0B,
    Equal = 0x0C,
    VolumeOverflow = 0x0D,
    Miscompare = 0x0E,
}

/// SCSI Request Sense. Additional Sense Code
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub struct AdditionalSenseCode {
    /// Additional Sense Code
    asc: u8,
    /// Additional Sense Code Qualifier
    ascq: u8,
    // TODO: SKSV, C/D BPV... をreportする場合は更に細分化して実装する
}

#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum AdditionalSenseCodeType {
    NoAdditionalSenseInformation,
    NotReadyCauseNotReportable,
    NotReadyInProcessOfBecomingReady,
    NotReadyManualInterventionRequired,
    NotReadyLogicalUnitNotReadyOperationInProgress,
    NotReadyLogicalUnitOffline,
    NotReadyMaintenanceMode,
    HardwareErrorGeneral,
    HardwareErrorTapeDrive,
    HardwareErrorCartridgeAccessPort,
    HardwareErrorEmbeddedSoftware,
    HardwareErrorMediaLoadEjectFailed,
    IllegalRequestInvalidFieldInCommandInfoUnit,
    IllegalRequestParameterLengthError,
    IllegalRequestInvalidCommand,
    IllegalRequestInvalidElement,
    IllegalRequestInvalidFieldInCdb,
    IllegalRequestLogicalUnitNotSupported,
    IllegalRequestInParameters,
    AbortedCommandLogicalUnitCommunicationFailure,
    AbortedCommandLogicalUnitCommunicationTimeout,
    AbortedCommandMechaicalPositioningError,
    AbortedCommandCommandPhaseError,
    AbortedCommandDataPhaseError,
    AbortedCommandCommandOverlapError,
}

impl AdditionalSenseCodeType {
    pub fn to_code(self) -> AdditionalSenseCode {
        match self {
            AdditionalSenseCodeType::NoAdditionalSenseInformation => AdditionalSenseCode {
                asc: 0x00,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::NotReadyCauseNotReportable => AdditionalSenseCode {
                asc: 0x04,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::NotReadyInProcessOfBecomingReady => AdditionalSenseCode {
                asc: 0x04,
                ascq: 0x01,
            },
            AdditionalSenseCodeType::NotReadyManualInterventionRequired => AdditionalSenseCode {
                asc: 0x04,
                ascq: 0x03,
            },
            AdditionalSenseCodeType::NotReadyLogicalUnitNotReadyOperationInProgress => {
                AdditionalSenseCode {
                    asc: 0x04,
                    ascq: 0x07,
                }
            }
            AdditionalSenseCodeType::NotReadyLogicalUnitOffline => AdditionalSenseCode {
                asc: 0x04,
                ascq: 0x12,
            },
            AdditionalSenseCodeType::NotReadyMaintenanceMode => AdditionalSenseCode {
                asc: 0x04,
                ascq: 0x81,
            },
            AdditionalSenseCodeType::HardwareErrorGeneral => AdditionalSenseCode {
                asc: 0x40,
                ascq: 0x01,
            },
            AdditionalSenseCodeType::HardwareErrorTapeDrive => AdditionalSenseCode {
                asc: 0x40,
                ascq: 0x02,
            },
            AdditionalSenseCodeType::HardwareErrorCartridgeAccessPort => AdditionalSenseCode {
                asc: 0x40,
                ascq: 0x03,
            },
            AdditionalSenseCodeType::HardwareErrorEmbeddedSoftware => AdditionalSenseCode {
                asc: 0x44,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::HardwareErrorMediaLoadEjectFailed => AdditionalSenseCode {
                asc: 0x53,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::IllegalRequestInvalidFieldInCommandInfoUnit => {
                AdditionalSenseCode {
                    asc: 0x24,
                    ascq: 0x00,
                }
            }
            AdditionalSenseCodeType::IllegalRequestParameterLengthError => AdditionalSenseCode {
                asc: 0x1a,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::IllegalRequestInvalidCommand => AdditionalSenseCode {
                asc: 0x20,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::IllegalRequestInvalidElement => AdditionalSenseCode {
                asc: 0x21,
                ascq: 0x01,
            },
            AdditionalSenseCodeType::IllegalRequestInvalidFieldInCdb => AdditionalSenseCode {
                asc: 0x24,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::IllegalRequestLogicalUnitNotSupported => AdditionalSenseCode {
                asc: 0x25,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::IllegalRequestInParameters => AdditionalSenseCode {
                asc: 0x26,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::AbortedCommandLogicalUnitCommunicationFailure => {
                AdditionalSenseCode {
                    asc: 0x08,
                    ascq: 0x00,
                }
            }
            AdditionalSenseCodeType::AbortedCommandLogicalUnitCommunicationTimeout => {
                AdditionalSenseCode {
                    asc: 0x08,
                    ascq: 0x01,
                }
            }
            AdditionalSenseCodeType::AbortedCommandMechaicalPositioningError => {
                AdditionalSenseCode {
                    asc: 0x15,
                    ascq: 0x01,
                }
            }
            AdditionalSenseCodeType::AbortedCommandCommandPhaseError => AdditionalSenseCode {
                asc: 0x4a,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::AbortedCommandDataPhaseError => AdditionalSenseCode {
                asc: 0x4b,
                ascq: 0x00,
            },
            AdditionalSenseCodeType::AbortedCommandCommandOverlapError => AdditionalSenseCode {
                asc: 0x4e,
                ascq: 0x00,
            },

            _ => {
                crate::unreachable!();
                AdditionalSenseCode { asc: 0, ascq: 0 }
            }
        }
    }
}

/// SCSI Inquiry command structure
pub const REQUEST_SENSE_DATA_SIZE: usize = 20;

/// SCSI Inquiry data structure
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub struct RequestSenseData {
    /// 0: Valid, 1: Invalid.  set to 0
    pub valid: bool,
    /// set to 0x70. returns only current error
    pub error_code: u8,
    /// set to 0x00.
    pub segment_number: u8,
    /// Sense key
    pub sense_key: SenseKey,
    /// set to 0x00
    pub information: u32,
    /// set to 0x0c
    pub additional_sense_length: u8,
    /// set to 0x00
    pub command_specific_information: u32,
    pub additional_sense_code: u8,
    pub additional_sense_code_qualifier: u8,
    pub field_replaceable_unit_code: u8,

    pub sksv: bool,
    pub cd: bool,
    pub bpv: bool,
    pub bit_pointer: u8,
    pub field_pointer: u16,
    pub reserved: u16,
}

impl RequestSenseData {
    pub fn new() -> Self {
        Self {
            valid: false,
            error_code: 0x70,
            segment_number: 0,
            sense_key: SenseKey::NoSense,
            information: 0,
            additional_sense_length: 0x0c,
            command_specific_information: 0,
            additional_sense_code: 0,
            additional_sense_code_qualifier: 0,
            field_replaceable_unit_code: 0,
            sksv: false,
            cd: false,
            bpv: false,
            bit_pointer: 0,
            field_pointer: 0,
            reserved: 0,
        }
    }

    pub fn from(sense_key: SenseKey, additional_sense_code: AdditionalSenseCodeType) -> Self {
        let asc = additional_sense_code.to_code();
        Self {
            valid: false,
            error_code: 0x70,
            segment_number: 0,
            sense_key,
            information: 0,
            additional_sense_length: 0x0c,
            command_specific_information: 0,
            additional_sense_code: asc.asc,
            additional_sense_code_qualifier: asc.ascq,
            field_replaceable_unit_code: 0,
            sksv: false,
            cd: false,
            bpv: false,
            bit_pointer: 0,
            field_pointer: 0,
            reserved: 0,
        }
    }

    /// Set additional sense code
    pub fn set_additional_sense_code(&mut self, code: AdditionalSenseCode) {
        self.additional_sense_code = code.asc;
        self.additional_sense_code_qualifier = code.ascq;
    }

    pub fn into_data(self) -> [u8; REQUEST_SENSE_DATA_SIZE] {
        let mut buf = [0u8; REQUEST_SENSE_DATA_SIZE];
        self.prepare_to_buf(&mut buf);
        buf
    }

    /// Prepare data to buffer
    pub fn prepare_to_buf(&self, buf: &mut [u8]) {
        crate::assert!(buf.len() >= REQUEST_SENSE_DATA_SIZE);

        buf[0] = ((self.valid as u8) << 7) | (self.error_code & 0x7f);
        buf[1] = self.segment_number;
        buf[2] = (self.sense_key as u8) & 0xf;
        BigEndian::write_u32(&mut buf[3..7], self.information);
        buf[7] = self.additional_sense_length;
        BigEndian::write_u32(&mut buf[8..12], self.command_specific_information);
        buf[12] = self.additional_sense_code;
        buf[13] = self.additional_sense_code_qualifier;
        buf[14] = self.field_replaceable_unit_code;
        buf[15] = ((self.sksv as u8) << 7)
            | ((self.cd as u8) << 6)
            | ((self.bpv as u8) << 5)
            | (self.bit_pointer & 0x7);
        BigEndian::write_u16(&mut buf[16..18], self.field_pointer);
        BigEndian::write_u16(&mut buf[18..20], self.reserved);
    }
}

/// SCSI Inquiry command structure
pub const INQUIRY_COMMAND_DATA_SIZE: usize = 36;
/// SCSI Inquiry command structure
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub struct InquiryCommandData {
    // byte0
    pub peripheral_qualifier: u8,
    pub peripheral_device_type: u8,
    // byte1
    pub rmb: bool,
    // byte2
    pub version: u8,
    // byte3
    pub aerc: bool,
    pub normaca: bool,
    pub hisup: bool,
    pub response_data_format: u8,
    // byte4
    pub additional_length: u8,
    // byte5
    pub sccs: bool,
    // byte6
    pub bque: bool,
    pub encserv: bool,
    pub vs0: bool,
    pub multip: bool,
    pub mchngr: bool,
    pub addr16: bool,
    // byte7
    pub reladr: bool,
    pub wbus16: bool,
    pub sync: bool,
    pub linked: bool,
    pub cmdque: bool,
    pub vs1: bool,
    // byte8-15
    pub vendor_id: [u8; 8],
    // byte16-31
    pub product_id: [u8; 16],
    // byte32-35
    pub product_revision_level: [u8; 4],
}

impl InquiryCommandData {
    pub fn new(vendor_id: [u8; 8], product_id: [u8; 16], product_revision_level: [u8; 4]) -> Self {
        Self {
            peripheral_qualifier: 0,
            peripheral_device_type: 0,
            rmb: true,
            version: 0x4,
            aerc: false,
            normaca: false,
            hisup: false,
            response_data_format: 0x2,
            additional_length: 0x1f,
            sccs: false,
            bque: false,
            encserv: false,
            vs0: false,
            multip: false,
            mchngr: false,
            addr16: false,
            reladr: false,
            wbus16: false,
            sync: false,
            linked: false,
            cmdque: false,
            vs1: false,
            vendor_id,
            product_id,
            product_revision_level,
        }
    }

    pub fn to_data(self) -> [u8; INQUIRY_COMMAND_DATA_SIZE] {
        let mut buf = [0u8; INQUIRY_COMMAND_DATA_SIZE];
        self.prepare_to_buf(&mut buf);
        buf
    }

    pub fn prepare_to_buf(&self, buf: &mut [u8]) {
        crate::assert!(buf.len() >= INQUIRY_COMMAND_DATA_SIZE);

        buf[0] = (self.peripheral_qualifier << 5) | (self.peripheral_device_type & 0x1f);
        buf[1] = ((self.rmb as u8) << 7);
        buf[2] = self.version;
        buf[3] = ((self.aerc as u8) << 7)
            | ((self.normaca as u8) << 5)
            | ((self.hisup as u8) << 4)
            | (self.response_data_format & 0xf);
        buf[4] = self.additional_length;
        buf[5] = ((self.sccs as u8) << 0x1);
        buf[6] = ((self.bque as u8) << 7)
            | ((self.encserv as u8) << 6)
            | ((self.vs0 as u8) << 5)
            | ((self.multip as u8) << 4)
            | ((self.mchngr as u8) << 3)
            | ((self.addr16 as u8) << 1);
        buf[7] = ((self.reladr as u8) << 7)
            | ((self.wbus16 as u8) << 6)
            | ((self.sync as u8) << 5)
            | ((self.linked as u8) << 4)
            | ((self.cmdque as u8) << 1)
            | (self.vs1 as u8);
        buf[8..16].copy_from_slice(&self.vendor_id);
        buf[16..32].copy_from_slice(&self.product_id);
        buf[32..36].copy_from_slice(&self.product_revision_level);
    }
}

/// SCSI Read Capacity command structure
pub const READ_FORMAT_CAPACITIES_DATA_SIZE: usize = 12;

/// SCSI Read Capacity command structure
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub struct ReadFormatCapacitiesData {
    pub capacity_list_length: u32,
    pub num_blocks: u32,
    pub descriptor_type: u8,
    pub block_length: u32,
}

impl ReadFormatCapacitiesData {
    pub fn new(num_blocks: u32, block_length: u32) -> Self {
        Self {
            capacity_list_length: 1,
            num_blocks,
            descriptor_type: 2, // formatted media
            block_length,
        }
    }

    pub fn to_data(self) -> [u8; READ_FORMAT_CAPACITIES_DATA_SIZE] {
        let mut buf = [0u8; READ_FORMAT_CAPACITIES_DATA_SIZE];
        self.prepare_to_buf(&mut buf);
        buf
    }

    pub fn prepare_to_buf(&self, buf: &mut [u8]) {
        crate::assert!(buf.len() >= READ_FORMAT_CAPACITIES_DATA_SIZE);
        // CapacityList Header
        BigEndian::write_u32(&mut buf[0..4], self.capacity_list_length);
        // Current/Maximum Capacity Descriptor
        BigEndian::write_u32(&mut buf[4..8], self.num_blocks);
        buf[8] = self.descriptor_type & 0x3;
        // Block Length fieldは3byteしかないので、上位1byteはCopyしない
        let mut block_length = [0u8; 4];
        BigEndian::write_u32(&mut block_length, self.block_length);
        buf[9..12].copy_from_slice(&block_length[1..4]);
    }
}

/// SCSI Read Capacity command length
pub const READ_CAPACITY_16_DATA_SIZE: usize = 8;

/// SCSI Read Capacity command structure
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub struct ReadCapacityData {
    pub last_lba: u32,
    pub block_length: u32,
}

impl ReadCapacityData {
    pub fn new(last_lba: u32, block_length: u32) -> Self {
        Self {
            last_lba,
            block_length,
        }
    }

    pub fn to_data(self) -> [u8; READ_CAPACITY_16_DATA_SIZE] {
        let mut buf = [0u8; READ_CAPACITY_16_DATA_SIZE];
        self.prepare_to_buf(&mut buf);
        buf
    }

    pub fn prepare_to_buf(&self, buf: &mut [u8]) {
        crate::assert!(buf.len() >= READ_CAPACITY_16_DATA_SIZE);
        BigEndian::write_u32(&mut buf[0..4], self.last_lba);
        BigEndian::write_u32(&mut buf[4..8], self.block_length);
    }
}

/// SCSI Mode Sense 6 command length
pub const MODE_SENSE_6_DATA_SIZE: usize = 4;

/// SCSI Mode Sense 6 command structure
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub struct ModeSense6Data {
    pub mode_data_length: u8,
    pub medium_type: u8,
    pub device_specific_parameter: u8,
    pub block_descriptor_length: u8,
}

impl ModeSense6Data {
    pub fn new() -> Self {
        Self {
            mode_data_length: 0x03,
            medium_type: 0,
            device_specific_parameter: 0,
            block_descriptor_length: 0,
        }
    }

    pub fn to_data(self) -> [u8; MODE_SENSE_6_DATA_SIZE] {
        let mut buf = [0u8; MODE_SENSE_6_DATA_SIZE];
        self.prepare_to_buf(&mut buf);
        buf
    }

    pub fn prepare_to_buf(&self, buf: &mut [u8]) {
        crate::assert!(buf.len() >= MODE_SENSE_6_DATA_SIZE);
        buf[0] = self.mode_data_length;
        buf[1] = self.medium_type;
        buf[2] = self.device_specific_parameter;
        buf[3] = self.block_descriptor_length;
    }
}

/// SCSI Reade 10 command length
pub const READ_10_DATA_SIZE: usize = 10;

/// SCSI Mode Sense 10 command structure
#[derive(Copy, Clone, PartialEq, Eq, defmt::Format)]
pub struct Read10Command {
    /// byte0: Operation Code (0x28)
    pub op_code: u8,
    /// byte1: Read Protect
    /// TODO: RDPROTECT fieldの表に従って実装
    pub rdprotect: u8,
    /// byte1: Disable Page Out
    /// 0: Page Out is enabled, 1: Page Out is disabled (Data is not cached)
    pub dpo: bool,
    /// byte1: FUA (Force Unit Access)
    /// 0: Normal, 1: FUA (Data is forced to be written to the medium)
    pub fua: bool,
    /// byte1: Read After Read Capable
    /// 0: Normal, 1: Read After Read Capable (Data is read after the read command)
    pub rarc: bool,
    /// byte2-5: Logical Block Address
    pub lba: u32,
    /// byte6: Group Number
    pub group_number: u8,
    /// byte7-8: Transfer Length
    pub transfer_length: u16,
    /// byte9: Normal ACA
    /// 0: Normal, 1: Normal ACA (ACA is enabled)
    pub naca: bool,
    /// byte9: Link
    /// 0: Normal, 1: Link (The command is linked)
    pub link: bool,
    /// byte9: Flag
    /// 0: Normal, 1: Flag (The command is flagged)
    pub flag: bool,
    /// byte9: Vendor Specific
    pub vendor_specific: u8,
}

impl Read10Command {
    pub fn new(lba: u32, transfer_length: u16) -> Self {
        Self {
            op_code: 0x28,
            rdprotect: 0,
            dpo: false,
            fua: false,
            rarc: false,
            lba,
            group_number: 0,
            transfer_length,
            naca: false,
            link: false,
            flag: false,
            vendor_specific: 0,
        }
    }

    pub fn from_data(data: &[u8]) -> Self {
        crate::assert!(data.len() >= READ_10_DATA_SIZE);
        Self {
            op_code: data[0],
            rdprotect: (data[1] >> 5) & 0x7,
            dpo: (data[1] & 0x10) != 0,
            fua: (data[1] & 0x08) != 0,
            rarc: (data[1] & 0x04) != 0,
            lba: BigEndian::read_u32(&data[2..6]),
            group_number: data[6] & 0x1f,
            transfer_length: BigEndian::read_u16(&data[7..9]),
            naca: (data[9] & 0x80) != 0,
            link: (data[9] & 0x40) != 0,
            flag: (data[9] & 0x20) != 0,
            vendor_specific: data[9] & 0x1f,
        }
    }
}
