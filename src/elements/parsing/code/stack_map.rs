use crate::{
    elements::{
        class_parser::{ClassFileParsingError, ClassFileParsingResult},
        method::{StackMapFrame, VerificationTypeInfo},
        parsing::constant_pool::ConstantPool,
    },
    utils::{read_u16, read_u8},
};

impl StackMapFrame {
    pub fn parse<R>(
        reader: &mut R,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<StackMapFrame>
    where
        R: std::io::Read,
    {
        let frame_type = read_u8(reader)?;
        let result = match frame_type {
            0..=63 => Self::SameFrame {
                offset_delta: frame_type as u16,
            },
            64..=127 => {
                Self::SameLocals1StackItemFrame(VerificationTypeInfo::parse(reader, constant_pool)?)
            }
            247 => {
                let offset_delta = read_u16(reader)?;
                let stack = VerificationTypeInfo::parse(reader, constant_pool)?;
                Self::Semantics1StackItemFrameExtended(offset_delta, stack)
            }
            248..=250 => {
                let chop_count = 251 - frame_type;
                let offset_delta = read_u16(reader)?;
                Self::ChopFrame {
                    chop_count,
                    offset_delta,
                }
            }
            251 => {
                let offset_delta = read_u16(reader)?;
                Self::SameFrameExtended { offset_delta }
            }
            252..=254 => {
                let offset_delta = read_u16(reader)?;
                let locals_count = frame_type - 251;
                let mut locals = Vec::with_capacity(locals_count as usize);
                for _ in 0..locals_count {
                    let local = VerificationTypeInfo::parse(reader, constant_pool)?;
                    locals.push(local);
                }
                Self::AppendFrame {
                    offset_delta,
                    locals,
                }
            }
            255 => {
                let offset_delta = read_u16(reader)?;
                let locals_count = read_u16(reader)?;
                let mut locals = Vec::with_capacity(locals_count as usize);
                for _ in 0..locals_count {
                    let local = VerificationTypeInfo::parse(reader, constant_pool)?;
                    locals.push(local);
                }
                let stacks_count = read_u16(reader)?;
                let mut stack = Vec::with_capacity(stacks_count as usize);
                for _ in 0..stacks_count {
                    let stack_element = VerificationTypeInfo::parse(reader, constant_pool)?;
                    stack.push(stack_element)
                }
                Self::FullFrame {
                    offset_delta,
                    locals,
                    stack,
                }
            }
            _ => Err(ClassFileParsingError::UnknownStackMapFrameType(frame_type))?,
        };
        Ok(result)
    }
}