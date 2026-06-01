use std::fmt;
use std::marker::PhantomData;

use crate::bridge::{ffi, sources as bridge_sources};
use crate::style::GeoJson;

/// Non-owning reference to a style source.
///
/// This reference is valid only for the lifetime of the borrowed [`Style`](crate::Style)
/// handle that created it.
#[derive(Debug)]
#[non_exhaustive]
pub enum SourceRef<'a> {
    /// A GeoJSON source.
    GeoJson(GeoJsonRef<'a>),
    /// A source that exists but is not yet supported by this crate.
    Unsupported(UnsupportedSourceRef<'a>),
}

/// Non-owning mutable reference to a style source.
///
/// This reference is valid only for the lifetime of the borrowed [`Style`](crate::Style)
/// handle that created it.
#[derive(Debug)]
#[non_exhaustive]
pub enum SourceRefMut<'a> {
    /// A GeoJSON source.
    GeoJson(GeoJsonRefMut<'a>),
    /// A source that exists but is not yet supported by this crate.
    Unsupported(UnsupportedSourceRefMut<'a>),
}

/// A non-owning reference to a GeoJSON style source.
pub struct GeoJsonRef<'a> {
    source: *const bridge_sources::GeoJSONSource,
    _lifetime: PhantomData<&'a ffi::MapRenderer>,
}

impl<'a> GeoJsonRef<'a> {
    pub(crate) fn from_raw(source: *const bridge_sources::GeoJSONSource) -> Option<Self> {
        (!source.is_null()).then_some(Self { source, _lifetime: PhantomData })
    }

    /// Returns the underlying raw MapLibre Native GeoJSON source pointer.
    #[must_use]
    pub fn as_raw(&self) -> *const bridge_sources::GeoJSONSource {
        self.source
    }
}

impl fmt::Debug for GeoJsonRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeoJsonRef").field("source", &self.source).finish_non_exhaustive()
    }
}

/// A non-owning mutable reference to a GeoJSON style source.
pub struct GeoJsonRefMut<'a> {
    source: *mut bridge_sources::GeoJSONSource,
    _lifetime: PhantomData<&'a mut ffi::MapRenderer>,
}

impl<'a> GeoJsonRefMut<'a> {
    pub(crate) fn from_raw(source: *mut bridge_sources::GeoJSONSource) -> Option<Self> {
        (!source.is_null()).then_some(Self { source, _lifetime: PhantomData })
    }

    /// Sets the GeoJSON data for this source.
    pub fn set_geojson(&mut self, geojson: &GeoJson) {
        unsafe { bridge_sources::setGeoJsonPtr(self.source, geojson.as_inner()) };
    }

    /// Returns the underlying raw MapLibre Native GeoJSON source pointer.
    #[must_use]
    pub fn as_raw(&self) -> *mut bridge_sources::GeoJSONSource {
        self.source
    }
}

impl fmt::Debug for GeoJsonRefMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeoJsonRefMut").field("source", &self.source).finish_non_exhaustive()
    }
}

/// A non-owning reference to an unsupported style source.
pub struct UnsupportedSourceRef<'a> {
    source: *const ffi::CxxSource,
    _lifetime: PhantomData<&'a ffi::MapRenderer>,
}

impl<'a> UnsupportedSourceRef<'a> {
    pub(crate) fn from_raw(source: *const ffi::CxxSource) -> Option<Self> {
        (!source.is_null()).then_some(Self { source, _lifetime: PhantomData })
    }

    /// Returns the underlying raw MapLibre Native source pointer.
    #[must_use]
    pub fn as_raw(&self) -> *const ffi::CxxSource {
        self.source
    }
}

impl fmt::Debug for UnsupportedSourceRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnsupportedSourceRef").field("source", &self.source).finish_non_exhaustive()
    }
}

/// A non-owning mutable reference to an unsupported style source.
pub struct UnsupportedSourceRefMut<'a> {
    source: *mut ffi::CxxSource,
    _lifetime: PhantomData<&'a mut ffi::MapRenderer>,
}

impl<'a> UnsupportedSourceRefMut<'a> {
    pub(crate) fn from_raw(source: *mut ffi::CxxSource) -> Option<Self> {
        (!source.is_null()).then_some(Self { source, _lifetime: PhantomData })
    }

    /// Returns the underlying raw MapLibre Native source pointer.
    #[must_use]
    pub fn as_raw(&self) -> *mut ffi::CxxSource {
        self.source
    }
}

impl fmt::Debug for UnsupportedSourceRefMut<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnsupportedSourceRefMut")
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}
