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

    pub sksv: u8,
    pub cd: u8,
    pub bpv: u8,
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
            sksv: 0,
            cd: 0,
            bpv: 0,
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
            sksv: 0,
            cd: 0,
            bpv: 0,
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

        buf[0] = (if self.valid { 0 } else { 0x80 }) | (self.error_code & 0x7f);
        buf[1] = self.segment_number;
        buf[2] = (self.sense_key as u8) & 0xf;
        BigEndian::write_u32(&mut buf[3..7], self.information);
        buf[7] = self.additional_sense_length;
        BigEndian::write_u32(&mut buf[8..12], self.command_specific_information);
        buf[12] = self.additional_sense_code;
        buf[13] = self.additional_sense_code_qualifier;
        buf[14] = self.field_replaceable_unit_code;
        buf[15] = ((self.sksv & 0x1) << 7)
            | ((self.cd & 0x1) << 6)
            | ((self.bpv & 0x1) << 5)
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
    pub rmb: u8,
    // byte2
    pub version: u8,
    // byte3
    pub aerc: u8,
    pub normaca: u8,
    pub hisup: u8,
    pub response_data_format: u8,
    // byte4
    pub additional_length: u8,
    // byte5
    pub sccs: u8,
    // byte6
    pub bque: u8,
    pub encserv: u8,
    pub vs0: u8,
    pub multip: u8,
    pub mchngr: u8,
    pub addr16: u8,
    // byte7
    pub reladr: u8,
    pub wbus16: u8,
    pub sync: u8,
    pub linked: u8,
    pub cmdque: u8,
    pub vs1: u8,
    // byte8-15
    pub vendor_id: [u8; 8],
    // byte16-31
    pub product_id: [u8; 16],
    // byte32-35
    pub product_revision_level: [u8; 4],
}

impl InquiryCommandData {
    pub fn new() -> Self {
        Self {
            peripheral_qualifier: 0,
            peripheral_device_type: 0,
            rmb: 0x1,
            version: 0x4,
            aerc: 0,
            normaca: 0,
            hisup: 0,
            response_data_format: 0x2,
            additional_length: 0x1f,
            sccs: 0,
            bque: 0,
            encserv: 0,
            vs0: 0,
            multip: 0,
            mchngr: 0,
            addr16: 0,
            reladr: 0,
            wbus16: 0,
            sync: 0,
            linked: 0,
            cmdque: 0,
            vs1: 0,
            vendor_id: *b"broccoli",
            product_id: *b"wipeseals devapp",
            product_revision_level: *b"0001",
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
        buf[1] = (self.rmb << 7);
        buf[2] = self.version;
        buf[3] = ((self.aerc & 0x1) << 7)
            | ((self.normaca & 0x1) << 5)
            | ((self.hisup & 0x1) << 4)
            | (self.response_data_format & 0xf);
        buf[4] = self.additional_length;
        buf[5] = (self.sccs << 0x1);
        buf[6] = ((self.bque & 0x1) << 7)
            | ((self.encserv & 0x1) << 6)
            | ((self.vs0 & 0x1) << 5)
            | ((self.multip & 0x1) << 4)
            | ((self.mchngr & 0x1) << 3)
            | ((self.addr16 & 0x1) << 1);
        buf[7] = ((self.reladr & 0x1) << 7)
            | ((self.wbus16 & 0x1) << 6)
            | ((self.sync & 0x1) << 5)
            | ((self.linked & 0x1) << 4)
            | ((self.cmdque & 0x1) << 1)
            | (self.vs1 & 0x1);
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
