use std::fmt;
use std::ops::{Deref, DerefMut};

use semver::Version;

use crate::{Interface, Render, RenderOpts, World, ident::Ident};

/// A WIT package.
///
/// A package is a collection of interfaces and worlds. Packages additionally
/// have a unique identifier that affects generated components and uniquely
/// identifiers this particular package.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub struct Package {
    /// A unique name corresponding to this package.
    name: PackageName,

    /// World items
    items: Vec<PackageItem>,
}

impl Package {
    /// Create a new instance of `Package`.
    pub fn new(name: PackageName) -> Self {
        Self {
            name,
            items: vec![],
        }
    }

    pub fn name(&self) -> &PackageName {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<PackageName>) {
        self.name = name.into();
    }

    /// Add an `Interface` to the package
    pub fn interface(&mut self, interface: Interface) {
        self.items.push(PackageItem::Interface(interface))
    }

    /// Add a `World` to the package
    pub fn world(&mut self, world: World) {
        self.items.push(PackageItem::World(world))
    }

    pub fn item(&mut self, item: impl Into<PackageItem>) {
        self.items.push(item.into());
    }

    pub fn items(&self) -> &[PackageItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<PackageItem> {
        &mut self.items
    }

    /// Render the items of the package, without any package declaration.
    fn render_items(&self, f: &mut fmt::Formatter<'_>, opts: &RenderOpts) -> fmt::Result {
        for item in &self.items {
            write!(f, "\n")?;
            match item {
                PackageItem::Interface(interface) => {
                    if let Some(docs) = &interface.docs {
                        docs.render(f, opts)?;
                    }
                    write!(f, "{}interface {} {{", opts.spaces(), interface.name)?;
                    if !interface.uses.is_empty() || !interface.items.is_empty() {
                        write!(f, "\n")?;
                        interface.uses.render(f, &opts.indent())?;
                        interface.items.render(f, &opts.indent())?;
                        write!(f, "{}}}\n", opts.spaces())?;
                    } else {
                        write!(f, "}}\n")?;
                    }
                }
                PackageItem::World(world) => {
                    world.render(f, opts)?;
                }
            }
        }
        Ok(())
    }
}

impl Render for Package {
    fn render(&self, f: &mut fmt::Formatter<'_>, opts: &RenderOpts) -> fmt::Result {
        write!(f, "{}package {};\n", opts.spaces(), self.name)?;
        self.render_items(f, opts)
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.render(f, &RenderOpts::default())
    }
}

/// A variant of [`Package`] that's rendered with a nested syntax.
pub struct NestedPackage(Package);

/// Provide associated functions explicitly.
/// [`Package`] instance methods are provided via the [`Deref`] and [`DerefMut`] traits.
impl NestedPackage {
    /// Create a new instance of `NestedPackage`.
    pub fn new(name: PackageName) -> Self {
        Self(Package::new(name))
    }
}

impl Deref for NestedPackage {
    type Target = Package;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NestedPackage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Render for NestedPackage {
    fn render(&self, f: &mut fmt::Formatter<'_>, opts: &RenderOpts) -> fmt::Result {
        write!(f, "{}package {} {{\n", opts.spaces(), self.0.name)?;
        self.0.render_items(f, &opts.indent())?;
        write!(f, "{}}}\n", opts.spaces())
    }
}

impl fmt::Display for NestedPackage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.render(f, &RenderOpts::default())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum PackageItem {
    Interface(Interface),
    World(World),
}

/// A structure used to keep track of the name of a package, containing optional
/// information such as a namespace and version information.
///
/// This is directly encoded as an "ID" in the binary component representation
/// with an interfaced tacked on as well.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub struct PackageName {
    /// A namespace such as `wasi` in `wasi:foo/bar`
    /// or `the:nested:namespace` in `the:nested:namespace:package/item`.
    /// Each namespace part is followed by a colon.
    /// A package namespace must include at least one part.
    namespace: Vec<String>,
    /// The (possibly nested) kebab-name of this package, such as `foo` in `wasi:foo/bar`
    /// or `the/nested/package` in `namespace:the/nested/package/item`.
    /// Name parts are separated by slashes.
    /// A package name must include at least one part.
    name: Vec<Ident>,
    /// Optional major/minor version information.
    version: Option<Version>,
}

impl PackageName {
    /// Create a new instance of `PackageName`
    pub fn new(
        namespace: impl IntoIterator<Item = String>,
        name: impl IntoIterator<Item = Ident>,
        version: Option<Version>,
    ) -> Self {
        Self {
            namespace: namespace.into_iter().collect(),
            name: name.into_iter().collect(),
            version,
        }
    }

    pub fn namespace(&self) -> &Vec<String> {
        &self.namespace
    }

    pub fn set_namespace(&mut self, namespace: impl IntoIterator<Item = String>) {
        self.namespace = namespace.into_iter().collect();
    }

    pub fn name(&self) -> &Vec<Ident> {
        &self.name
    }

    pub fn set_name(&mut self, name: impl IntoIterator<Item = Ident>) {
        self.name = name.into_iter().collect();
    }

    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }

    pub fn set_version(&mut self, version: Option<impl Into<Version>>) {
        self.version = version.map(|v| v.into())
    }
}

impl From<PackageName> for String {
    fn from(name: PackageName) -> String {
        name.to_string()
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_package_name(
            f,
            self.namespace.iter().map(String::as_str),
            self.name.iter().map(Ident::raw_name),
            &self.version,
        )
    }
}

/// Helper function to format package names or interface IDs,
/// which follow the same pattern: `namespace:parts:name/parts[/interface-name][@version]`.
///
/// In the case of interface IDs,
/// simply append the interface name to the end of the package name iterator.
pub(crate) fn fmt_package_name<
    'a,
    W: fmt::Write,
    NS: Iterator<Item = &'a str>,
    N: Iterator<Item = &'a str>,
    V: fmt::Display,
>(
    dst: &mut W,
    namespace: NS,
    mut name: N,
    version: &Option<V>,
) -> fmt::Result {
    for namespace_part in namespace {
        write!(dst, "{namespace_part}:")?;
    }

    if let Some(name_part) = name.next() {
        dst.write_str(name_part)?;
        for name_part in name {
            write!(dst, "/{name_part}")?;
        }
    }

    if let Some(version) = version {
        write!(dst, "@{version}")?;
    }

    Ok(())
}
