use crate::{
    elements::{
        class_file::{ClassFileParsingError, ClassFileParsingResult},
        constant_pool::ConstantPool,
        fields::ConstantValue,
    },
    utils::{read_u16, read_u32, read_u8},
};

use super::Attribute;

#[derive(Debug)]
pub enum ElementValue {
    Constant(ConstantValue),
    EnumConstant {
        type_name: String,
        const_name: String,
    },
    Class {
        return_descriptor: String,
    },
    AnnotationInterface(Annotation),
    Array(Vec<ElementValue>),
}
impl ElementValue {
    fn parse<R>(
        reader: &mut R,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<ElementValue>
    where
        R: std::io::Read,
    {
        let tag = read_u8(reader)?;
        let const_value_index = read_u16(reader)?;

        macro_rules! read_constant {
            ($constant_type:path) => {{
                let $constant_type(value) = constant_pool.get_constant_value(const_value_index)? else {
                                return Err(ClassFileParsingError::MidmatchedConstantPoolTag);
                            };
                Ok(Self::Constant($constant_type(value)))
            }};
        }
        match tag as char {
            'B' | 'C' | 'I' | 'S' | 'Z' => read_constant!(ConstantValue::Integer),
            'D' => read_constant!(ConstantValue::Double),
            'F' => read_constant!(ConstantValue::Float),
            'J' => read_constant!(ConstantValue::Long),
            's' => read_constant!(ConstantValue::String),
            'e' => {
                let enum_type_idx = read_u16(reader)?;
                let type_name = constant_pool.get_string(enum_type_idx)?;
                let const_name_idx = read_u16(reader)?;
                let const_name = constant_pool.get_string(const_name_idx)?;
                Ok(Self::EnumConstant {
                    type_name,
                    const_name,
                })
            }
            'c' => {
                let class_info_idx = read_u16(reader)?;
                let return_descriptor = constant_pool.get_string(class_info_idx)?;
                Ok(Self::Class { return_descriptor })
            }
            '@' => Annotation::parse(reader, constant_pool).map(Self::AnnotationInterface),
            '[' => {
                let num_values = read_u16(reader)?;
                let mut values = Vec::with_capacity(num_values as usize);
                for _ in 0..num_values {
                    values.push(Self::parse(reader, constant_pool)?);
                }
                Ok(Self::Array(values))
            }
            _ => Err(ClassFileParsingError::InvalidElementValueTag(tag)),
        }
    }
}

#[derive(Debug)]
pub struct Annotation {
    pub annotation_type_desc: String,
    pub element_value_pairs: Vec<(String, ElementValue)>,
}

impl Annotation {
    fn parse<R>(
        reader: &mut R,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<Annotation>
    where
        R: std::io::Read,
    {
        let type_idx = read_u16(reader)?;
        let annotation_type_desc = constant_pool.get_string(type_idx)?;
        let num_element_value_pairs = read_u16(reader)?;
        let mut element_value_pairs = Vec::with_capacity(num_element_value_pairs as usize);
        for _ in 0..num_element_value_pairs {
            let element_name_idx = read_u16(reader)?;
            let element_name = constant_pool.get_string(element_name_idx)?;
            let element_value = ElementValue::parse(reader, constant_pool)?;
            element_value_pairs.push((element_name, element_value));
        }
        Ok(Annotation {
            annotation_type_desc,
            element_value_pairs,
        })
    }
}

#[derive(Debug)]
pub enum TargetInfo {
    TypeParameter(u8),
    SuperType(u16),
    TypeParameterBound(u8, u8),
    Empty,
    FormalParameter(u8),
    Throws(u16),
    LocalVar(Vec<(u16, u16, u16)>),
    Catch(u16),
    Offset(u16),
    TypeArgument(u16, u8),
}

#[derive(Debug)]
pub enum TypePathKind {
    Array = 0x00,
    Nested = 0x01,
    Bound = 0x02,
    TypeArgument = 0x03,
}

#[derive(Debug)]
pub struct TypePathElement {
    pub kind: TypePathKind,
    pub argument_index: u8,
}
impl TypePathElement {
    fn parse<R>(reader: &mut R) -> ClassFileParsingResult<TypePathElement>
    where
        R: std::io::Read,
    {
        let kind = match read_u8(reader)? {
            0x00 => TypePathKind::Array,
            0x01 => TypePathKind::Nested,
            0x02 => TypePathKind::Bound,
            0x03 => TypePathKind::TypeArgument,
            _ => Err(ClassFileParsingError::InvalidTypePathKind)?,
        };
        let argument_index = read_u8(reader)?;
        Ok(Self {
            kind,
            argument_index,
        })
    }
}

#[derive(Debug)]
pub struct TypeAnnotation {
    pub target_info: TargetInfo,
    pub target_path: Vec<TypePathElement>,
    pub type_index: u16,
    pub element_value_pairs: Vec<(String, ElementValue)>,
}
impl TypeAnnotation {
    fn parse<R>(reader: &mut R, constant_pool: &ConstantPool) -> ClassFileParsingResult<Self>
    where
        R: std::io::Read,
    {
        let target_type = read_u8(reader)?;
        let target_info = match target_type {
            0x00 | 0x01 => TargetInfo::TypeParameter(read_u8(reader)?),
            0x10 => TargetInfo::SuperType(read_u16(reader)?),
            0x11 | 0x12 => TargetInfo::TypeParameterBound(read_u8(reader)?, read_u8(reader)?),
            0x13..=0x15 => TargetInfo::Empty,
            0x16 => TargetInfo::FormalParameter(read_u8(reader)?),
            0x17 => TargetInfo::Throws(read_u16(reader)?),
            0x40 | 0x41 => {
                let table_length = read_u16(reader)?;
                let mut table = Vec::with_capacity(table_length as usize);
                for _ in 0..table_length {
                    let start_pc = read_u16(reader)?;
                    let length = read_u16(reader)?;
                    let index = read_u16(reader)?;
                    table.push((start_pc, length, index));
                }
                TargetInfo::LocalVar(table)
            }
            0x42 => TargetInfo::Catch(read_u16(reader)?),
            0x43..=0x46 => TargetInfo::Offset(read_u16(reader)?),
            0x47..=0x4B => TargetInfo::TypeArgument(read_u16(reader)?, read_u8(reader)?),
            _ => Err(ClassFileParsingError::InvalidTargetType(target_type))?,
        };
        let mut target_path = Vec::new();
        let path_length = read_u8(reader)?;
        for _ in 0..path_length {
            let type_path_element = TypePathElement::parse(reader)?;
            target_path.push(type_path_element);
        }
        let type_index = read_u16(reader)?;
        let num_element_value_pairs = read_u16(reader)?;
        let mut element_value_pairs = Vec::with_capacity(num_element_value_pairs as usize);
        for _ in 0..num_element_value_pairs {
            let element_name_idx = read_u16(reader)?;
            let element_name = constant_pool.get_string(element_name_idx)?;
            let element_value = ElementValue::parse(reader, constant_pool)?;
            element_value_pairs.push((element_name, element_value));
        }
        Ok(TypeAnnotation {
            target_info,
            target_path,
            type_index,
            element_value_pairs,
        })
    }
}

impl Attribute {
    pub(super) fn parse_annotations<R>(
        reader: &mut R,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<Vec<Annotation>>
    where
        R: std::io::Read,
    {
        let _attribute_length = read_u32(reader)?;
        let num_annotations = read_u16(reader)?;
        let mut annotations = Vec::with_capacity(num_annotations as usize);
        for _ in 0..num_annotations {
            let annotation = Annotation::parse(reader, constant_pool)?;
            annotations.push(annotation);
        }

        Ok(annotations)
    }

    pub(super) fn parse_parameter_annotations<R>(
        reader: &mut R,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<Vec<Vec<Annotation>>>
    where
        R: std::io::Read,
    {
        let _attribute_length = read_u32(reader)?;
        let num_parameters = read_u8(reader)?;
        let mut parameter_annotations = Vec::with_capacity(num_parameters as usize);
        for _ in 0..num_parameters {
            let par_annotations = Self::parse_annotations(reader, constant_pool)?;
            parameter_annotations.push(par_annotations);
        }
        Ok(parameter_annotations)
    }

    pub(super) fn parse_type_annotations<R>(
        reader: &mut R,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<Vec<TypeAnnotation>>
    where
        R: std::io::Read,
    {
        let _attribute_length = read_u32(reader)?;
        let num_annotations = read_u16(reader)?;
        let mut annotations = Vec::with_capacity(num_annotations as usize);
        for _ in 0..num_annotations {
            let type_annotation = TypeAnnotation::parse(reader, constant_pool)?;
            annotations.push(type_annotation);
        }
        Ok(annotations)
    }

    pub(super) fn parse_annotation_default<R>(
        reader: &mut R,
        constant_pool: &ConstantPool,
    ) -> ClassFileParsingResult<Self>
    where
        R: std::io::Read,
    {
        let _attribute_length = read_u32(reader)?;
        let value = ElementValue::parse(reader, constant_pool)?;
        Ok(Self::AnnotationDefault(value))
    }
}
