use crate::{
    elements::{
        class_parser::{ClassFileParsingError, ClassFileParsingResult},
        field::{ConstantValue, PrimitiveType},
        instruction::Instruction,
        parsing::constant_pool::ConstantPool,
        references::MethodReference,
    },
    utils::{read_i16, read_i32, read_i8, read_u16, read_u8},
};

impl Instruction {
    pub fn parse_code(
        bytes: Vec<u8>,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<Vec<Self>> {
        let mut cursor = std::io::Cursor::new(bytes);
        let mut instructions = Vec::new();
        loop {
            if let Some(instruction) = Instruction::parse(&mut cursor, constant_pool)? {
                instructions.push(instruction);
            } else {
                break;
            }
        }
        Ok(instructions)
    }

    pub(crate) fn parse(
        reader: &mut std::io::Cursor<Vec<u8>>,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<Option<Self>> {
        let opcode = match read_u8(reader) {
            Ok(it) => it,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    return Ok(None);
                } else {
                    return Err(ClassFileParsingError::MalformedClassFile);
                }
            }
        };
        let instruction = match opcode {
            0x32 => Self::AALoad,
            0x53 => Self::AAStore,
            0x01 => Self::AConstNull,
            0x19 => Self::ALoad(read_u8(reader)?),
            0x2a => Self::ALoad0,
            0x2b => Self::ALoad1,
            0x2c => Self::ALoad2,
            0x2d => Self::ALoad3,
            0xbd => {
                let index = read_u16(reader)?;
                let array_type = constant_pool.get_array_type_ref(&index)?;
                Self::ANewArray(array_type)
            }
            0xb0 => Self::AReturn,
            0xbe => Self::ArrayLength,
            0x3a => Self::AStore(read_u8(reader)?),
            0x4b => Self::AStore0,
            0x4c => Self::AStore1,
            0x4d => Self::AStore2,
            0x4e => Self::AStore3,
            0xbf => Self::AThrow,
            0x33 => Self::BALoad,
            0x54 => Self::BAStore,
            0x10 => Self::BiPush(read_u8(reader)?),
            0x34 => Self::CALoad,
            0x55 => Self::CAStore,
            0xc0 => Self::CheckCast(read_u16(reader)?),
            0x90 => Self::D2F,
            0x8e => Self::D2I,
            0x8f => Self::D2L,
            0x63 => Self::DAdd,
            0x31 => Self::DALoad,
            0x52 => Self::DAStore,
            0x98 => Self::DCmpG,
            0x97 => Self::DCmpL,
            0x0e => Self::DConst0,
            0x0f => Self::DConst1,
            0x6f => Self::DDiv,
            0x18 => Self::DLoad(read_u8(reader)?),
            0x26 => Self::DLoad0,
            0x27 => Self::DLoad1,
            0x28 => Self::DLoad2,
            0x29 => Self::DLoad3,
            0x6b => Self::DMul,
            0x77 => Self::DNeg,
            0x73 => Self::DRem,
            0xaf => Self::DReturn,
            0x39 => Self::DStore(read_u8(reader)?),
            0x47 => Self::DStore0,
            0x48 => Self::DStore1,
            0x49 => Self::DStore2,
            0x4a => Self::DStore3,
            0x67 => Self::DSub,
            0x59 => Self::Dup,
            0x5a => Self::DupX1,
            0x5b => Self::DupX2,
            0x5c => Self::Dup2,
            0x5d => Self::Dup2X1,
            0x5e => Self::Dup2X2,
            0x8d => Self::F2D,
            0x8b => Self::F2I,
            0x8c => Self::F2L,
            0x62 => Self::FAdd,
            0x30 => Self::FALoad,
            0x51 => Self::FAStore,
            0x96 => Self::FCmpG,
            0x95 => Self::FCmpL,
            0x0b => Self::FConst0,
            0x0c => Self::FConst1,
            0x0d => Self::FConst2,
            0x6e => Self::FDiv,
            0x17 => Self::FLoad(read_u8(reader)?),
            0x22 => Self::FLoad0,
            0x23 => Self::FLoad1,
            0x24 => Self::FLoad2,
            0x25 => Self::FLoad3,
            0x6a => Self::FMul,
            0x76 => Self::FNeg,
            0x72 => Self::FRem,
            0xae => Self::FReturn,
            0x38 => Self::FStore(read_u8(reader)?),
            0x43 => Self::FStore0,
            0x44 => Self::FStore1,
            0x45 => Self::FStore2,
            0x46 => Self::FStore3,
            0x66 => Self::FSub,
            0xb4 => {
                let index = read_u16(reader)?;
                let field = constant_pool.get_field_ref(&index)?;
                Self::GetField(field)
            }
            0xb2 => {
                let index = read_u16(reader)?;
                let field = constant_pool.get_field_ref(&index)?;
                Self::GetStatic(field)
            }
            0xc8 => Self::GotoW(read_i32(reader)?),
            0x91 => Self::I2B,
            0x92 => Self::I2C,
            0x87 => Self::I2D,
            0x86 => Self::I2F,
            0x85 => Self::I2L,
            0x93 => Self::I2S,
            0x60 => Self::IAdd,
            0x2e => Self::IALoad,
            0x7e => Self::IAnd,
            0x4f => Self::IAStore,
            0x02 => Self::IConstM1,
            0x03 => Self::IConst0,
            0x04 => Self::IConst1,
            0x05 => Self::IConst2,
            0x06 => Self::IConst3,
            0x07 => Self::IConst4,
            0x08 => Self::IConst5,
            0x6c => Self::IDiv,
            0xa5 => Self::IfACmpEq(read_i16(reader)?),
            0xa6 => Self::IfACmpNe(read_i16(reader)?),
            0x9f => Self::IfICmpEq(read_i16(reader)?),
            0xa0 => Self::IfICmpNe(read_i16(reader)?),
            0xa1 => Self::IfICmpLt(read_i16(reader)?),
            0xa2 => Self::IfICmpGe(read_i16(reader)?),
            0xa3 => Self::IfICmpGt(read_i16(reader)?),
            0xa4 => Self::IfICmpLe(read_i16(reader)?),
            0x99 => Self::IfEq(read_i16(reader)?),
            0x9a => Self::IfNe(read_i16(reader)?),
            0x9b => Self::IfLt(read_i16(reader)?),
            0x9c => Self::IfGe(read_i16(reader)?),
            0x9d => Self::IfGt(read_i16(reader)?),
            0x9e => Self::IfLe(read_i16(reader)?),
            0xc7 => Self::IfNonNull(read_i16(reader)?),
            0xc6 => Self::IfNull(read_i16(reader)?),
            0x84 => Self::IInc(read_u8(reader)?, read_i8(reader)?),
            0x15 => Self::ILoad(read_u8(reader)?),
            0x1a => Self::ILoad0,
            0x1b => Self::ILoad1,
            0x1c => Self::ILoad2,
            0x1d => Self::ILoad3,
            0x68 => Self::IMul,
            0x74 => Self::INeg,
            0xc1 => Self::InstanceOf(read_u16(reader)?),
            0xba => {
                let index = read_u16(reader)?;
                let method_handle = constant_pool.get_method_handle(&index)?;
                let zeros = read_u16(reader)?;
                if zeros != 0 {
                    Err(ClassFileParsingError::MalformedClassFile)?
                }
                Self::InvokeDynamic(method_handle)
            }
            0xb9 => {
                let index = read_u16(reader)?;
                let MethodReference::Interface(method_ref) = constant_pool.get_method_ref(&index)? else {
                    Err(ClassFileParsingError::MalformedClassFile)?
                };
                let count = read_u8(reader)?;
                let zero = read_u8(reader)?;
                if zero != 0 {
                    Err(ClassFileParsingError::MalformedClassFile)?
                }
                Self::InvokeInterface(method_ref, count)
            }
            0xb7 => {
                let index = read_u16(reader)?;
                let method_ref = constant_pool.get_method_ref(&index)?;
                Self::InvokeSpecial(method_ref)
            }
            0xb8 => {
                let index = read_u16(reader)?;
                let method_ref = constant_pool.get_method_ref(&index)?;
                Self::InvokeStatic(method_ref)
            }
            0xb6 => {
                let index = read_u16(reader)?;
                let method_ref = constant_pool.get_method_ref(&index)?;
                Self::InvokeVirtual(method_ref)
            }
            0x80 => Self::IOr,
            0x70 => Self::IRem,
            0xac => Self::IReturn,
            0x78 => Self::IShl,
            0x7a => Self::IShr,
            0x36 => Self::IStore(read_u8(reader)?),
            0x3b => Self::IStore0,
            0x3c => Self::IStore1,
            0x3d => Self::IStore2,
            0x3e => Self::IStore3,
            0x64 => Self::ISub,
            0x7c => Self::IUShr,
            0x82 => Self::IXor,
            0xa8 => Self::Jsr(read_i16(reader)?),
            0xc9 => Self::JsrW(read_i32(reader)?),
            0x8a => Self::L2D,
            0x89 => Self::L2F,
            0x88 => Self::L2I,
            0x61 => Self::LAdd,
            0x2f => Self::LALoad,
            0x7f => Self::LAnd,
            0x50 => Self::LAStore,
            0x94 => Self::LCmp,
            0x09 => Self::LConst0,
            0x0a => Self::LConst1,
            0x12 => {
                let index = read_u8(reader)? as u16;
                let constant = match constant_pool.get_constant_value(&index)? {
                    ConstantValue::Long(_) | ConstantValue::Double(_) => {
                        Err(ClassFileParsingError::MalformedClassFile)?
                    }
                    // TODO: Check dynamically computed symbolic reference
                    it @ _ => it,
                };
                Self::Ldc(constant)
            }
            0x13 => {
                let index = read_u16(reader)?;
                let constant = match constant_pool.get_constant_value(&index)? {
                    ConstantValue::Long(_) | ConstantValue::Double(_) => {
                        Err(ClassFileParsingError::MalformedClassFile)?
                    }
                    // TODO: Check dynamically computed symbolic reference
                    it @ _ => it,
                };
                Self::LdcW(constant)
            }
            0x14 => {
                let index = read_u16(reader)?;
                let constant = match constant_pool.get_constant_value(&index)? {
                    it @ ConstantValue::Long(_) => it,
                    it @ ConstantValue::Double(_) => it,
                    // TODO: Check dynamically computed symbolic reference
                    _ => Err(ClassFileParsingError::MalformedClassFile)?,
                };
                Self::Ldc2W(constant)
            }
            0x6d => Self::LDiv,
            0x16 => Self::LLoad(read_u8(reader)?),
            0x1e => Self::LLoad0,
            0x1f => Self::LLoad1,
            0x20 => Self::LLoad2,
            0x21 => Self::LLoad3,
            0x69 => Self::LMul,
            0x75 => Self::LNeg,
            0xab => {
                while reader.position() % 4 != 0 {
                    let _padding_byte = read_u8(reader)?;
                }
                let default = read_i32(reader)?;
                let npairs = read_i32(reader)?;
                let mut match_offsets = Vec::with_capacity(npairs as usize);
                for _ in 0..npairs {
                    let match_value = read_i32(reader)?;
                    let offset = read_i32(reader)?;
                    match_offsets.push((match_value, offset));
                }
                Self::LookupSwitch {
                    default,
                    match_offsets,
                }
            }
            0xaa => {
                while reader.position() % 4 != 0 {
                    let _padding_byte = read_u8(reader)?;
                }
                let default = read_i32(reader)?;
                let low = read_i32(reader)?;
                let high = read_i32(reader)?;
                let offset_count = high - low + 1;
                let mut jump_offsets = Vec::with_capacity(offset_count as usize);
                for _ in 0..offset_count {
                    let offset = read_i32(reader)?;
                    jump_offsets.push(offset);
                }
                Self::TableSwitch {
                    default,
                    low,
                    high,
                    jump_offsets,
                }
            }
            0x81 => Self::LOr,
            0x71 => Self::LRem,
            0xad => Self::LReturn,
            0x79 => Self::LShl,
            0x7b => Self::LShr,
            0x37 => Self::LStore(read_u8(reader)?),
            0x3f => Self::LStore0,
            0x40 => Self::LStore1,
            0x41 => Self::LStore2,
            0x42 => Self::LStore3,
            0x65 => Self::LSub,
            0x7d => Self::LUShr,
            0x83 => Self::LXor,
            0xc2 => Self::MonitorEnter,
            0xc3 => Self::MonitorExit,
            0xc5 => {
                let index = read_u16(reader)?;
                let array_type = constant_pool.get_array_type_ref(&index)?;
                Self::MultiANewArray(array_type, read_u8(reader)?)
            }
            0xbb => {
                let index = read_u16(reader)?;
                let class_ref = constant_pool.get_class_ref(&index)?;
                Self::New(class_ref)
            }
            0xbc => {
                let type_id = read_u8(reader)?;
                let arr_type = match type_id {
                    4 => PrimitiveType::Boolean,
                    5 => PrimitiveType::Char,
                    6 => PrimitiveType::Float,
                    7 => PrimitiveType::Double,
                    8 => PrimitiveType::Byte,
                    9 => PrimitiveType::Short,
                    10 => PrimitiveType::Int,
                    11 => PrimitiveType::Long,
                    _ => Err(ClassFileParsingError::MalformedClassFile)?
                };
                Self::NewArray(arr_type)
            }
            0x00 => Self::Nop,
            0x57 => Self::Pop,
            0x58 => Self::Pop2,
            0xb5 => {
                let index = read_u16(reader)?;
                let field = constant_pool.get_field_ref(&index)?;
                Self::PutField(field)
            }
            0xb3 => {
                let index = read_u16(reader)?;
                let field = constant_pool.get_field_ref(&index)?;
                Self::PutStatic(field)
            }
            0xa9 => Self::Ret(read_u8(reader)?),
            0xb1 => Self::Return,
            0x35 => Self::SALoad,
            0x56 => Self::SAStore,
            0x11 => Self::SiPush(read_u16(reader)?),
            0x5f => Self::Swap,
            0xc4 => {
                let wide_opcode = read_u8(reader)?;
                match wide_opcode {
                    0x15 => Self::WideILoad(read_u16(reader)?),
                    0x16 => Self::WideLLoad(read_u16(reader)?),
                    0x17 => Self::WideFLoad(read_u16(reader)?),
                    0x18 => Self::WideDLoad(read_u16(reader)?),
                    0x19 => Self::WideALoad(read_u16(reader)?),
                    0x36 => Self::WideIStore(read_u16(reader)?),
                    0x37 => Self::WideLStore(read_u16(reader)?),
                    0x38 => Self::WideFStore(read_u16(reader)?),
                    0x39 => Self::WideDStore(read_u16(reader)?),
                    0x3a => Self::WideAStore(read_u16(reader)?),
                    0xa9 => Self::WideRet(read_u16(reader)?),
                    0x84 => Self::WideIInc(read_u16(reader)?, read_i16(reader)?),
                    _ => Err(ClassFileParsingError::UnexpectedOpCode)?,
                }
            }
            _ => Err(ClassFileParsingError::UnexpectedOpCode)?,
        };
        Ok(Some(instruction))
    }
}