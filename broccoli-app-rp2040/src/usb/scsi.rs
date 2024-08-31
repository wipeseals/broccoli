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
    ReadCapacity16_10 = 0x25,
    Read10 = 0x28,
    Write10 = 0x2A,
    Verify10 = 0x2F,
}

/// SCSI Inquiry command structure
pub struct RequestSenseCommand {
    pub operation_code: u8,
    pub allocation_length: u8,
    pub control: u8,
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
    pub fn to_code(&self) -> AdditionalSenseCode {
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

    pub fn to_data(&self) -> [u8; REQUEST_SENSE_DATA_SIZE] {
        let mut data = [0u8; REQUEST_SENSE_DATA_SIZE];
        data[0] = (if self.valid { 0 } else { 0x80 }) | (self.error_code & 0x7f);
        data[1] = self.segment_number;
        data[2] = (self.sense_key as u8) & 0xf;
        data[3..7].copy_from_slice(&self.information.to_be_bytes());
        data[7] = self.additional_sense_length;
        data[8..12].copy_from_slice(&self.command_specific_information.to_be_bytes());
        data[12] = self.additional_sense_code;
        data[13] = self.additional_sense_code_qualifier;
        data[14] = self.field_replaceable_unit_code;
        data[15] = ((self.sksv & 0x1) << 7)
            | ((self.cd & 0x1) << 6)
            | ((self.bpv & 0x1) << 5)
            | (self.bit_pointer & 0x7);
        data[16..18].copy_from_slice(&self.field_pointer.to_be_bytes());
        data[18..20].copy_from_slice(&self.reserved.to_be_bytes());
        data
    }

    /// Prepare data to buffer
    pub fn prepare_to_buf(&self, buf: &mut [u8]) {
        crate::assert!(buf.len() >= REQUEST_SENSE_DATA_SIZE);

        buf[0] = (if self.valid { 0 } else { 0x80 }) | (self.error_code & 0x7f);
        buf[1] = self.segment_number;
        buf[2] = (self.sense_key as u8) & 0xf;
        buf[3..7].copy_from_slice(&self.information.to_be_bytes());
        buf[7] = self.additional_sense_length;
        buf[8..12].copy_from_slice(&self.command_specific_information.to_be_bytes());
        buf[12] = self.additional_sense_code;
        buf[13] = self.additional_sense_code_qualifier;
        buf[14] = self.field_replaceable_unit_code;
        buf[15] = ((self.sksv & 0x1) << 7)
            | ((self.cd & 0x1) << 6)
            | ((self.bpv & 0x1) << 5)
            | (self.bit_pointer & 0x7);
        buf[16..18].copy_from_slice(&self.field_pointer.to_be_bytes());
        buf[18..20].copy_from_slice(&self.reserved.to_be_bytes());
    }
}
