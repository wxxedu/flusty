use flusty_parse::rust::types::RsFn;

use crate::conversion::{ToDart, ToDartFfi};

const DART_TEMPLATE: &str = r#"
import 'dart:ffi' as ffi;
import 'dart:io' show Platform, Directory;
import 'package:path/path.dart' as path;

#TYPE_DEFS#

class #MODULE_NAME# {
  static ffi.DynamicLibrary? _lib;

  static ffi.DynamicLibrary get lib {
    if (_lib != null) {
      return _lib!;
    }
    var libPath = path.join(
      Directory.current.path,
      #PATH_TO_LIB#
      #LIB_NAME#.so
    );
    if (Platform.isMacOS) {
        libPath = path.join(
            libPath, 
            #PATH_TO_LIB#
            #LIB_NAME#.dylib
        );
    } else if (Platform.isWindows) {
        libPath = path.join(
            libPath, 
            #PATH_TO_LIB#
            #LIB_NAME#.dll
        );
    }
    _lib = ffi.DynamicLibrary.open(libPath);
    return _lib!;
  }

#FUNCTIONS#
}

#TYPES#
"#;

pub fn fill_dart_template(
    type_defs: impl AsRef<str>,
    module_name: impl AsRef<str>,
    path_to_lib: impl AsRef<str>,
    lib_name: impl AsRef<str>,
    functions: impl AsRef<str>,
    types: impl AsRef<str>,
) -> String {
    DART_TEMPLATE
        .replace("#TYPE_DEFS#", type_defs.as_ref())
        .replace("#MODULE_NAME#", module_name.as_ref())
        .replace("#PATH_TO_LIB#", path_to_lib.as_ref())
        .replace("#LIB_NAME#", lib_name.as_ref())
        .replace("#FUNCTIONS#", functions.as_ref())
        .replace("#TYPES#", types.as_ref())
}

const DART_TYPE_DEF_TEMPLATE: &str = r#"
typedef #TYPE_NAME# = #TYPE# Function(#ARGS#);
"#;

pub fn fill_dart_type_def_template(
    type_name: impl AsRef<str>,
    type_: impl AsRef<str>,
    args: impl AsRef<str>,
) -> String {
    DART_TYPE_DEF_TEMPLATE
        .replace("#TYPE_NAME#", type_name.as_ref())
        .replace("#TYPE#", type_.as_ref())
        .replace("#ARGS#", args.as_ref())
}

const DART_STRUCT_TEMPLATE: &str = r#"
final class #STRUCT_NAME# extends ffi.Struct {
    #FIELDS#
}
"#;

pub fn fill_dart_struct_template(
    struct_name: impl AsRef<str>,
    fields: impl AsRef<str>,
) -> String {
    DART_STRUCT_TEMPLATE
        .replace("#STRUCT_NAME#", struct_name.as_ref())
        .replace("#FIELDS#", fields.as_ref())
}

const DART_FIELD_TEMPLATE: &str = r#"
#ANNOTATION#
external #TYPE# get #FIELD_NAME#;
"#;

pub fn fill_dart_field_template(
    annotation: impl AsRef<str>,
    type_: impl AsRef<str>,
    field_name: impl AsRef<str>,
) -> String {
    DART_FIELD_TEMPLATE
        .replace("#ANNOTATION#", annotation.as_ref())
        .replace("#TYPE#", type_.as_ref())
        .replace("#FIELD_NAME#", field_name.as_ref())
}

const DART_UNION_TEMPLATE: &str = r#"
final class #UNION_NAME# extends ffi.Union {
    #FIELDS#
}
"#;

const FUNCTION_TEMPLATE: &str = r#"
final #DART_TYPE# #FUNCTION_NAME# = lib
    .lookup<ffi.NativeFunction<#NATIVE_TYPE#>>('#FUNCTION_NAME#')
    .asFunction();
"#;

pub fn fill_dart_union_template(
    union_name: impl AsRef<str>,
    fields: impl AsRef<str>,
) -> String {
    DART_UNION_TEMPLATE
        .replace("#UNION_NAME#", union_name.as_ref())
        .replace("#FIELDS#", fields.as_ref())
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct DartFileBuilder {
    type_defs: Vec<String>,
    module_name: Option<String>,
    path_to_lib: Option<Vec<String>>,
    lib_name: Option<String>,
    functions: Vec<String>,
    types: Vec<String>,
}

impl DartFileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_function(&mut self, f: &RsFn) {
        let dart_typedef = f.to_dart();
        let native_typedef = f.to_dart_ffi();
    }

    pub fn build(self) -> String {
        let type_defs = self.type_defs.join("\n");
        let module_name = self.module_name.unwrap();
        let path_to_lib = self.path_to_lib.unwrap().join("/");
        let lib_name = self.lib_name.unwrap();
        let functions = self.functions.join("\n");
        let types = self.types.join("\n");
        fill_dart_template(
            type_defs,
            module_name,
            path_to_lib,
            lib_name,
            functions,
            types,
        )
    }
}

