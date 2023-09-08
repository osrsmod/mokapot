use crate::utils::{read_u16, read_u32};

use super::{
    class::{Class, ClassAccessFlags, ClassVersion},
    field::Field,
    method::Method,
    parsing::{
        attribute::{Attribute, AttributeList},
        constant_pool::ParsingContext,
        error::ClassFileParsingError,
    },
};

pub struct ClassParser<'a> {
    reader: &'a mut dyn std::io::Read,
}

impl<'a> ClassParser<'a> {
    pub fn parse(mut self) -> Result<Class, ClassFileParsingError> {
        let reader = &mut self.reader;

        let magic = read_u32(reader)?;
        if magic != JAVA_CLASS_MAIGC {
            return Err(ClassFileParsingError::NotAClassFile);
        }
        let version = ClassVersion::parse(reader)?;
        let parsing_context = ParsingContext::parse(reader, &version)?;
        let access = read_u16(reader)?;
        let Some(access_flags) = ClassAccessFlags::from_bits(access) else {
            return Err(ClassFileParsingError::UnknownFlags(access, "class"));
        };
        let this_class_idx = read_u16(reader)?;
        let this_class = parsing_context.get_class_ref(&this_class_idx)?;
        let super_class_idx = read_u16(reader)?;
        let super_class = match super_class_idx {
            0 if this_class.binary_name == "java/lang/Object" => None,
            0 if access_flags.contains(ClassAccessFlags::MODULE) => None,
            0 => Err(ClassFileParsingError::MalformedClassFile(
                "Class must have a super type except for java/lang/Object or a module",
            ))?,
            it @ _ => Some(parsing_context.get_class_ref(&it)?),
        };

        let interfaces_count = read_u16(reader)?;
        let interfaces = (0..interfaces_count)
            .map(|_| {
                let interface_idx = read_u16(reader)?;
                parsing_context.get_class_ref(&interface_idx)
            })
            .collect::<Result<_, ClassFileParsingError>>()?;
        let fields_count = read_u16(reader)?;
        let fields = (0..fields_count)
            .into_iter()
            .map(|_| Field::parse(reader, &parsing_context))
            .collect::<Result<_, ClassFileParsingError>>()?;

        let methods_count = read_u16(reader)?;
        let methods = (0..methods_count)
            .map(|_| Method::parse(reader, &parsing_context))
            .collect::<Result<_, ClassFileParsingError>>()?;

        let attributes = AttributeList::parse(reader, &parsing_context)?;

        let mut may_remain: [u8; 1] = [0];
        let remain = reader.read(&mut may_remain)?;
        if remain == 1 {
            return Err(ClassFileParsingError::UnexpectedData);
        }

        let mut source_file = None;
        let mut inner_classes = None;
        let mut enclosing_method = None;
        let mut source_debug_extension = None;
        let mut rt_visible_anno = None;
        let mut rt_invisible_anno = None;
        let mut rt_visible_type_anno = None;
        let mut rt_invisible_type_anno = None;
        let mut bootstrap_methods = None;
        let mut module = None;
        let mut module_packages = None;
        let mut module_main_class = None;
        let mut nest_host = None;
        let mut nest_members = None;
        let mut permitted_subclasses = None;
        let mut is_synthetic = false;
        let mut is_deprecated = false;
        let mut signature = None;
        let mut record = None;
        for attr in attributes.into_iter() {
            match attr {
                Attribute::SourceFile(file_name) => source_file = Some(file_name),
                Attribute::InnerClasses(it) => inner_classes = Some(it),
                Attribute::EnclosingMethod(em) => enclosing_method = Some(em),
                Attribute::SourceDebugExtension(sde) => source_debug_extension = Some(sde),
                Attribute::RuntimeVisibleAnnotations(rv) => rt_visible_anno = Some(rv),
                Attribute::RuntimeInvisibleAnnotations(ri) => rt_invisible_anno = Some(ri),
                Attribute::RuntimeVisibleTypeAnnotations(rv) => rt_visible_type_anno = Some(rv),
                Attribute::RuntimeInvisibleTypeAnnotations(ri) => rt_invisible_type_anno = Some(ri),
                Attribute::BootstrapMethods(bm) => bootstrap_methods = Some(bm),
                Attribute::Module(m) => module = Some(m),
                Attribute::ModulePackages(mp) => module_packages = Some(mp),
                Attribute::ModuleMainClass(mmc) => module_main_class = Some(mmc),
                Attribute::NestHost(nh) => nest_host = Some(nh),
                Attribute::NestMembers(nm) => nest_members = Some(nm),
                Attribute::PermittedSubclasses(ps) => permitted_subclasses = Some(ps),
                Attribute::Synthetic => is_synthetic = true,
                Attribute::Deprecated => is_deprecated = true,
                Attribute::Signature(sig) => signature = Some(sig),
                Attribute::Record(rec) => record = Some(rec),
                it @ _ => Err(ClassFileParsingError::UnexpectedAttribute(
                    it.name(),
                    "class_file",
                ))?,
            }
        }
        Ok(Class {
            version,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            source_file,
            inner_classes: inner_classes.unwrap_or_default(),
            enclosing_method,
            source_debug_extension: source_debug_extension.unwrap_or_default(),
            runtime_visible_annotations: rt_visible_anno.unwrap_or_default(),
            runtime_invisible_annotations: rt_invisible_anno.unwrap_or_default(),
            runtime_visible_type_annotations: rt_visible_type_anno.unwrap_or_default(),
            runtime_invisible_type_annotations: rt_invisible_type_anno.unwrap_or_default(),
            bootstrap_methods: bootstrap_methods.unwrap_or_default(),
            module,
            module_packages: module_packages.unwrap_or_default(),
            module_main_class,
            nest_host,
            nest_members: nest_members.unwrap_or_default(),
            permitted_subclasses: permitted_subclasses.unwrap_or_default(),
            is_synthetic,
            is_deprecated,
            signature,
            record: record.unwrap_or_default(),
        })
    }
}

impl ClassVersion {
    /// Returns `true` if this class file is compiled with `--enable-preview`.
    pub fn is_preview_enabled(&self) -> bool {
        self.minor == 65535
    }
    fn parse<R>(reader: &mut R) -> std::io::Result<Self>
    where
        R: std::io::Read,
    {
        let minor = read_u16(reader)?;
        let major = read_u16(reader)?;
        Ok(Self { major, minor })
    }
}

const JAVA_CLASS_MAIGC: u32 = 0xCAFEBABE;

impl<'a> ClassParser<'a> {
    pub fn from_reader<'r, R>(reader: &'r mut R) -> ClassParser<'r>
    where
        R: std::io::Read,
    {
        ClassParser { reader }
    }
}
