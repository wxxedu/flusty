use std::{error::Error, ops::Add};

use flusty_parse::rust::types::{
    RsEnum, RsField, RsFn, RsPrimitive, RsStruct, RsType, RsVariant,
};

#[derive(Debug, Clone)]
pub enum ConversionError {
    UnsupportedType(RsType),
    MissingModuleName,
    MissingLibPath,
    MissingLibName,
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConversionError::UnsupportedType(ty) => {
                write!(f, "Type does not exist: {:?}", ty)
            }
            ConversionError::MissingModuleName => {
                write!(f, "Missing module name")
            }
            ConversionError::MissingLibPath => {
                write!(f, "Missing library path")
            }
            ConversionError::MissingLibName => {
                write!(f, "Missing library name")
            }
        }
    }
}

impl Error for ConversionError {}

pub trait Named {
    fn snake_case_name(&self) -> String {
        let name = self.camel_case_name();
        let mut name = name.chars();
        let mut result = String::new();
        while let Some(c) = name.next() {
            if c.is_uppercase() {
                if let Some(next) = name.next() {
                    if next.is_lowercase() {
                        result.push('_');
                    }
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }
        result
    }

    fn pascal_case_name(&self) -> String {
        let name = self.snake_case_name();
        // convert the snake case name to camel case
        let mut name = name.split('_').map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        });
        let first = name.next().unwrap();
        let rest = name.collect::<Vec<_>>().join("");
        format!("{}{}", first, rest)
    }

    fn camel_case_name(&self) -> String {
        let name = self.pascal_case_name();
        let mut chars = name.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_lowercase().chain(chars).collect(),
        }
    }
}

macro_rules! impl_named {
    (snake: $($t:ty),*) => {
        $(
            impl Named for $t {
                fn snake_case_name(&self) -> String {
                    self.name.clone()
                }
            }
        )*
    };
    (pascal: $($t:ty),*) => {
        $(
            impl Named for $t {
                fn pascal_case_name(&self) -> String {
                    self.name.clone()
                }
            }
        )*
    };
    (cammel: $($t:ty),*) => {
        $(
            impl Named for $t {
                fn camel_case_name(&self) -> String {
                    self.name.clone()
                }
            }
        )*
    };

}

impl_named!(pascal: RsStruct, RsEnum, RsVariant);
impl_named!(snake: RsField, RsFn);

pub trait ToDart {
    fn to_dart(&self) -> Result<String, ConversionError>;
}

pub trait DartField {
    /// If no issue occurs, returns a tuple of
    /// (ffi_arg_display, dart_arg_display, dart_field_display)
    fn display(
        &self,
        type_only: bool,
    ) -> Result<(String, String, String), ConversionError>;
}

impl DartField for RsField {
    fn display(
        &self,
        type_only: bool,
    ) -> Result<(String, String, String), ConversionError> {
        let (_, ffi_type, dart_type) = self.ty.name()?;
        let ffi_name = self.snake_case_name();
        let dart_name = self.camel_case_name();
        let dart_field_display =
            format!("@{}() external {} {};", ffi_type, dart_type, dart_name);
        if type_only {
            return Ok((ffi_type, dart_type, dart_field_display));
        }
        Ok((
            format!("{} {}", ffi_type, ffi_name),
            format!("{} {}", dart_type, dart_name),
            dart_field_display,
        ))
    }
}

pub trait DartType {
    /// If no issue occurs, returns a tuple of
    /// (native_type_name, ffi_type_name, dart_type_name)
    fn name(&self) -> Result<(String, String, String), ConversionError>;
}

impl DartType for RsType {
    fn name(&self) -> Result<(String, String, String), ConversionError> {
        match self {
            RsType::Primitive(p) => p.name(),
            RsType::Struct(s) => Ok((
                s.pascal_case_name(),
                s.pascal_case_name(),
                s.pascal_case_name(),
            )),
            RsType::Enum(e) => Ok((
                e.pascal_case_name(),
                e.pascal_case_name(),
                e.pascal_case_name(),
            )),
            RsType::Fn(f) => Ok((
                f.snake_case_name(),
                f.snake_case_name(),
                f.pascal_case_name(),
            )),
            RsType::Tuple(_) => todo!(),
            RsType::Pointer(p) => {
                let (native, ffi, dart) = p.name()?;
                Ok((
                    format!("*mut {}", native),
                    format!("ffi.Pointer<{}>", ffi),
                    format!("ffi.Pointer<{}>", dart),
                ))
            }
        }
    }
}

impl DartType for RsPrimitive {
    fn name(&self) -> Result<(String, String, String), ConversionError> {
        match self {
            RsPrimitive::I8 => Ok((
                "i8".to_string(),
                "ffi.Int8".to_string(),
                "int".to_string(),
            )),
            RsPrimitive::I16 => Ok((
                "i16".to_string(),
                "ffi.Int16".to_string(),
                "int".to_string(),
            )),
            RsPrimitive::I32 => Ok((
                "i32".to_string(),
                "ffi.Int32".to_string(),
                "int".to_string(),
            )),
            RsPrimitive::I64 => Ok((
                "i64".to_string(),
                "ffi.Int64".to_string(),
                "int".to_string(),
            )),
            RsPrimitive::I128 => Err(ConversionError::UnsupportedType(
                RsType::Primitive(RsPrimitive::I128),
            )),
            RsPrimitive::U8 => Ok((
                "u8".to_string(),
                "ffi.Uint8".to_string(),
                "int".to_string(),
            )),
            RsPrimitive::U16 => Ok((
                "u16".to_string(),
                "ffi.Uint16".to_string(),
                "int".to_string(),
            )),
            RsPrimitive::U32 => Ok((
                "u32".to_string(),
                "ffi.Uint32".to_string(),
                "int".to_string(),
            )),
            RsPrimitive::U64 => Err(ConversionError::UnsupportedType(
                RsType::Primitive(RsPrimitive::U64),
            )),
            RsPrimitive::U128 => Err(ConversionError::UnsupportedType(
                RsType::Primitive(RsPrimitive::U128),
            )),
            RsPrimitive::F32 => Ok((
                "f32".to_string(),
                "ffi.Float".to_string(),
                "double".to_string(),
            )),
            RsPrimitive::F64 => Ok((
                "f64".to_string(),
                "ffi.Double".to_string(),
                "double".to_string(),
            )),
            RsPrimitive::Bool => Ok((
                "bool".to_string(),
                "ffi.Uint8".to_string(),
                "bool".to_string(),
            )),
            RsPrimitive::Char => Ok((
                "char".to_string(),
                "ffi.Int32".to_string(),
                "String".to_string(),
            )),
            RsPrimitive::Str => Ok((
                "str".to_string(),
                "ffi.Pointer<ffi.Utf8>".to_string(),
                "String".to_string(),
            )),
            RsPrimitive::String => Ok((
                "String".to_string(),
                "ffi.Pointer<ffi.Utf8>".to_string(),
                "String".to_string(),
            )),
            RsPrimitive::Unit => Ok((
                "()".to_string(),
                "ffi.Void".to_string(),
                "void".to_string(),
            )),
        }
    }
}

pub trait DartFn {
    /// If no issue occurs, returns a tuple of
    /// (ffi_fn_typedef, dart_fn_typedef, fn_linker)
    fn typedef(&self) -> Result<(String, String, String), ConversionError>;
}

impl DartFn for RsFn {
    fn typedef(&self) -> Result<(String, String, String), ConversionError> {
        let native_name = self.snake_case_name();
        let ffi_type_name = self.snake_case_name();
        let dart_type_name = self.pascal_case_name();
        let name = self.camel_case_name();
        let ffi_args = self
            .args
            .iter()
            .map(|arg| {
                let (fii, _, _) = arg.display(true)?;
                Ok(fii)
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        let dart_args = self
            .args
            .iter()
            .map(|arg| {
                let (_, dart, _) = arg.display(true)?;
                Ok(dart)
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        let ffi_ret = self.ret.name()?.1;
        let dart_ret = self.ret.name()?.2;
        let ffi_typedef = format!(
            "typedef {} = {} Function({});",
            ffi_type_name, ffi_ret, ffi_args
        );
        let dart_typedef = format!(
            "typedef {} = {} Function({});",
            dart_type_name, dart_ret, dart_args
        );
        let fn_linker = format!(
            "final {} {} = dylib().lookup<ffi.NativeFunction<{}>>('{}').asFunction();",
            dart_type_name, name, ffi_type_name, native_name
        );
        Ok((ffi_typedef, dart_typedef, fn_linker))
    }
}

const DART_TEMPLATE: &str = r#"
import 'dart:ffi' as ffi;
import 'dart:io' show Platform, Directory;
import 'package:path/path.dart' as path;

#TYPE_DEFS#
final class #MODULE_NAME# {
    static ffi.DynamicLibrary? _dylib;

    static ffi.DynamicLibrary dylib() {
        if (_dylib != null) {
            return _dylib!;
        }
        var libraryPath = path.join(
            Directory.current.path, 
            #LIB_PATH#
            '#LIB_NAME#.so'
        );
        if (Platform.isMacOS) {
            libraryPath = path.join(
                Directory.current.path, 
                #LIB_PATH#
                '#LIB_NAME#.dylib'
            );
        } else if (Platform.isWindows) {
            libraryPath = path.join(
                Directory.current.path, 
                #LIB_PATH#
                '#LIB_NAME#.dll'
            );
        }
        _dylib = ffi.DynamicLibrary.open(libraryPath);
        return _dylib!;
    }

#FN_LINKERS#
}
"#;

#[derive(Debug, Default)]
pub struct DartFileBuilder {
    pub type_defs: Vec<String>,
    pub module_name: Option<String>,
    pub lib_path: Vec<String>,
    pub lib_name: Option<String>,
    pub fn_linkers: Vec<String>,
}

impl DartFileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fn(&mut self, f: &impl DartFn) -> Result<(), ConversionError> {
        let (ffi_typedef, dart_typedef, fn_linker) = f.typedef()?;
        self.type_defs.push(ffi_typedef);
        self.type_defs.push(dart_typedef);
        self.fn_linkers.push(fn_linker);
        Ok(())
    }

    pub fn set_module_name(&mut self, name: &str) {
        self.module_name = Some(name.into());
    }

    pub fn add_lib_path(&mut self, path: &str) {
        self.lib_path.push(path.into());
    }

    pub fn set_lib_path(&mut self, path: &[&str]) {
        self.lib_path = path.iter().map(|p| p.to_string()).collect();
        dbg!(&self.lib_path);
    }

    pub fn set_lib_name(&mut self, name: &str) {
        self.lib_name = Some(name.into());
    }

    pub fn build(&self) -> Result<String, ConversionError> {
        let module_name = self
            .module_name
            .clone()
            .ok_or_else(|| ConversionError::MissingModuleName)?;
        let lib_path = if !self.lib_path.is_empty() {
            self.lib_path
                .iter()
                .map(|p| format!("'{}'", p))
                .collect::<Vec<_>>()
                .join(", ")
                .add(", ")
        } else {
            "".to_string()
        };
        dbg!(&lib_path);
        let lib_name = self
            .lib_name
            .clone()
            .ok_or_else(|| ConversionError::MissingLibName)?;
        let type_defs = self.type_defs.join("\n");
        let fn_linkers = self.fn_linkers.join("\n");
        let template = DART_TEMPLATE
            .replace("#TYPE_DEFS#", &type_defs)
            .replace("#MODULE_NAME#", &module_name)
            .replace("#LIB_PATH#", &lib_path)
            .replace("#LIB_NAME#", &lib_name)
            .replace("#FN_LINKERS#", &fn_linkers);
        Ok(template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NamedObject {
        name: String,
    }

    impl Named for NamedObject {
        fn snake_case_name(&self) -> String {
            self.name.clone()
        }
    }

    #[test]
    fn test_dart_object() {
        let obj = NamedObject {
            name: "hello_world".into(),
        };
        assert_eq!(obj.pascal_case_name(), "HelloWorld");
        assert_eq!(obj.camel_case_name(), "helloWorld");
    }
}
