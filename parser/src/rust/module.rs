use std::{error::Error, fmt::Display};

use syn::{
    parse_file, Attribute, File, Item, ItemEnum, ItemFn, ItemMod, ItemStruct,
    Meta,
};

use super::types::{RsEnum, RsFn, RsStruct};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleError {
    MissingName,
    MissingPath,
    InvalidModule { name: String, path: String },
}

impl Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleError::MissingName => write!(f, "Missing name"),
            ModuleError::MissingPath => write!(f, "Missing path"),
            ModuleError::InvalidModule { name, path } => {
                write!(f, "Invalid module: {} in {}", name, path)
            }
        }
    }
}

impl Error for ModuleError {}

unsafe impl Send for Module {}

unsafe impl Sync for Module {}

#[derive(Debug, Clone, Default)]
pub struct Module {
    pub name: String,
    pub path: String,
    pub children: Vec<Module>,
    pub structs: Vec<RsStruct>,
    pub functions: Vec<RsFn>,
    pub enums: Vec<RsEnum>,
}

impl Module {
    pub fn builder<'a>(annotations: &'a [&'a str]) -> ModuleBuilder<'a> {
        ModuleBuilder::new(annotations)
    }
}

#[derive(Debug, Clone)]
pub struct ModuleBuilder<'a> {
    name: Option<String>,
    path: Option<String>,
    children: Vec<Module>,
    annotations: &'a [&'a str],
    structs: Vec<RsStruct>,
    functions: Vec<RsFn>,
    enums: Vec<RsEnum>,
}

impl<'a> ModuleBuilder<'a> {
    pub fn new(annotation: &'a [&'a str]) -> Self {
        log::info!(
            "Creating new module builder searching for {:?}",
            annotation
        );
        Self {
            name: None,
            path: None,
            children: vec![],
            annotations: annotation,
            structs: vec![],
            functions: vec![],
            enums: vec![],
        }
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        log::info!("Setting name to {}", name);
        self.name = Some(name);
        self
    }

    pub fn path(&mut self, path: String) -> &mut Self {
        log::info!("Setting path to {}", path);
        self.path = Some(path);
        self
    }

    fn read_module(&self) -> Result<File, ModuleError> {
        if self.name.is_none() {
            log::error!("Missing name");
            return Err(ModuleError::MissingName);
        }
        let name = self.name.as_ref().unwrap();
        if self.path.is_none() {
            log::error!("Missing path");
            return Err(ModuleError::MissingPath);
        }
        let path = self.path.as_ref().unwrap();
        let path_1 = format!("{}/{}.rs", path, name);
        let path_2 = format!("{}/{}.mod.rs", path, name);
        if let Ok(file) = std::fs::read_to_string(&path_1) {
            log::info!("Reading file {}", &path_1);
            let res =
                parse_file(&file).map_err(|_| ModuleError::InvalidModule {
                    name: self.name.as_ref().unwrap().clone(),
                    path: path.to_string(),
                })?;
            return Ok(res);
        }
        if let Ok(file) = std::fs::read_to_string(&path_2) {
            log::info!("Reading file {}", &path_2);
            let res =
                parse_file(&file).map_err(|_| ModuleError::InvalidModule {
                    name: self.name.as_ref().unwrap().clone(),
                    path: path.to_string(),
                })?;
            return Ok(res);
        }
        log::error!("Missing file {} or {}", &path_1, &path_2);
        Err(ModuleError::InvalidModule {
            name: self.name.as_ref().unwrap().clone(),
            path: path.to_string(),
        })
    }

    fn should_include(&mut self, attr: &Attribute) -> bool {
        match &attr.meta {
            Meta::Path(path) => {
                for annotation in self.annotations {
                    if path.is_ident(annotation) {
                        return true;
                    }
                }
                return false;
            }
            Meta::List(list) => {
                for annotation in self.annotations {
                    if list.path.is_ident(annotation) {
                        return true;
                    }
                }
                return false;
            }
            Meta::NameValue(_) => return false,
        }
    }

    fn handle_mod(&mut self, item: &ItemMod) -> Result<(), ModuleError> {
        match &item.vis {
            syn::Visibility::Public(_) => {
                log::info!("Found module {}", item.ident);
                let mut builder = ModuleBuilder::new(self.annotations);
                let res = builder
                    .name(item.ident.to_string())
                    .path(self.path.as_ref().unwrap().clone())
                    .data()?
                    .build();
                self.children.push(res);
                Ok(())
            }
            _ => {
                log::info!("Skipping module {}", item.ident);
                return Ok(());
            }
        }
    }

    fn handle_fn(&mut self, item: &ItemFn) -> Result<(), ModuleError> {
        match &item.vis {
            syn::Visibility::Public(_) => {
                for attr in &item.attrs {
                    if self.should_include(attr) {
                        log::info!("Adding function {}", item.sig.ident);
                        self.functions.push(item.clone().into());
                        return Ok(());
                    } else {
                        log::info!("Skipping function {}", item.sig.ident);
                    }
                }
                Ok(())
            }
            _ => {
                log::info!("Skipping function {}", item.sig.ident);
                return Ok(());
            }
        }
    }

    fn handle_struct(&mut self, item: &ItemStruct) -> Result<(), ModuleError> {
        match &item.vis {
            syn::Visibility::Public(_) => {
                for attr in &item.attrs {
                    if self.should_include(attr) {
                        log::info!("Adding struct {}", item.ident);
                        self.structs.push(item.clone().into());
                        return Ok(());
                    } else {
                        log::info!("Skipping struct {}", item.ident);
                    }
                }
                Ok(())
            }
            _ => {
                log::info!("Skipping struct {}", item.ident);
                return Ok(());
            }
        }
    }

    fn handle_enum(&mut self, item: &ItemEnum) -> Result<(), ModuleError> {
        match &item.vis {
            syn::Visibility::Public(_) => {
                for attr in &item.attrs {
                    if self.should_include(attr) {
                        log::info!("Adding enum {}", item.ident);
                        self.enums.push(item.clone().into());
                        return Ok(());
                    } else {
                        log::info!("Skipping enum {}", item.ident);
                    }
                }
                Ok(())
            }
            _ => {
                log::info!("Skipping enum {}", item.ident);
                return Ok(());
            }
        }
    }

    pub fn data(&mut self) -> Result<&mut Self, ModuleError> {
        let file = self.read_module()?;
        for item in &file.items {
            match item {
                Item::Mod(item) => self.handle_mod(&item)?,
                Item::Fn(f) => self.handle_fn(&f)?,
                Item::Struct(s) => self.handle_struct(&s)?,
                Item::Enum(e) => self.handle_enum(&e)?,
                _ => continue,
            }
        }
        Ok(self)
    }

    pub fn build(&self) -> Module {
        Module {
            name: self.name.clone().unwrap(),
            path: self.path.clone().unwrap(),
            children: self.children.clone(),
            structs: self.structs.clone(),
            functions: self.functions.clone(),
            enums: self.enums.clone(),
        }
    }
}
