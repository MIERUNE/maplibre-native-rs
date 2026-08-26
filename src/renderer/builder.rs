//! Image renderer configuration and builder

use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::rc::Rc;

use crate::bridge::ffi;
use crate::renderer::map_observer::MapObserverCallbacks;
use crate::renderer::{Continuous, ImageRenderer, MapMode, Static, Tile};
use crate::ResourceOptions;

/// Builder for configuring [`ImageRenderer`] instances
///
/// Each thread that constructs an [`ImageRenderer`] lazily creates one MapLibre
/// Native run loop. Renderers created on the same thread share that loop, while
/// renderers created on different threads use independent loops. A renderer must
/// be constructed and used on the same thread.
///
/// # Examples
///
/// ```
/// use maplibre_native::ImageRendererBuilder;
/// use std::num::NonZeroU32;
///
/// let renderer = ImageRendererBuilder::new()
///     .with_size(NonZeroU32::new(1024).unwrap(), NonZeroU32::new(768).unwrap())
///     .with_pixel_ratio(2.0)
///     .build_static_renderer();
/// ```
#[derive(Debug)]
pub struct ImageRendererBuilder {
    /// Image width in pixels
    width: NonZeroU32,
    /// Image height in pixels
    height: NonZeroU32,
    /// Pixel ratio for high-DPI displays
    pixel_ratio: f32,
    resource_options: Option<ResourceOptions>,
}

impl Default for ImageRendererBuilder {
    #[allow(clippy::missing_panics_doc, reason = "infallible")]
    fn default() -> Self {
        Self {
            width: NonZeroU32::new(512).unwrap(),
            height: NonZeroU32::new(512).unwrap(),
            pixel_ratio: 1.0,
            resource_options: None,
        }
    }
}

impl ImageRendererBuilder {
    /// Creates a new builder with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets image dimensions
    ///
    /// Default: `512` x `512`
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_size(mut self, width: NonZeroU32, height: NonZeroU32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets pixel ratio for high-DPI displays
    ///
    /// Default: `1.0`
    #[must_use]
    pub fn with_pixel_ratio(mut self, pixel_ratio: f32) -> Self {
        self.pixel_ratio = pixel_ratio;
        self
    }

    /// Set Resource Options
    #[must_use]
    #[allow(clippy::needless_pass_by_value, reason = "false positive")]
    pub fn with_resource_options(mut self, resource_options: ResourceOptions) -> Self {
        self.resource_options = Some(resource_options);
        self
    }

    /// Builds a static image renderer
    #[must_use]
    pub fn build_static_renderer(self) -> ImageRenderer<Static> {
        // TODO: Should the width/height be passed in here, or have another `build_static_with_size` method?
        ImageRenderer::new(MapMode::Static, self)
    }

    /// Builds a static image renderer, surfacing graphics-stack construction
    /// failures (for example an EGL display that cannot be initialized) as an
    /// error instead of a panic.
    ///
    /// # Errors
    ///
    /// Returns [`RendererBuildError`] when the native renderer cannot be
    /// constructed, typically because the platform graphics context cannot be
    /// created.
    pub fn try_build_static_renderer(
        self,
    ) -> Result<ImageRenderer<Static>, RendererBuildError> {
        ImageRenderer::try_new(MapMode::Static, self)
    }

    /// Builds a tile renderer
    #[must_use]
    pub fn build_tile_renderer(self) -> ImageRenderer<Tile> {
        // TODO: Is the width/height used for this mode?
        ImageRenderer::new(MapMode::Tile, self)
    }

    /// Fallible variant of [`build_tile_renderer`](Self::build_tile_renderer);
    /// see [`try_build_static_renderer`](Self::try_build_static_renderer).
    ///
    /// # Errors
    ///
    /// Returns [`RendererBuildError`] when the native renderer cannot be
    /// constructed.
    pub fn try_build_tile_renderer(self) -> Result<ImageRenderer<Tile>, RendererBuildError> {
        ImageRenderer::try_new(MapMode::Tile, self)
    }

    /// Builds a continuous renderer.
    ///
    /// Frames are host-driven: use [`ImageRenderer::<Continuous>::set_render_requested_callback`] to wake the
    /// host UI's display loop, then call [`ImageRenderer::<Continuous>::render_once`] for each scheduled frame.
    ///
    /// Use the `MapObserver` to react to signals from the map.
    ///
    /// # Breaking change
    ///
    /// This renderer no longer self-invalidates (MapLibre Native used to schedule
    /// its own renders on update). The signature is unchanged, so existing callers
    /// keep compiling but stop producing frames until they drive
    /// [`render_once`](ImageRenderer::<Continuous>::render_once) themselves.
    #[must_use]
    pub fn build_continuous_renderer(self) -> ImageRenderer<Continuous> {
        ImageRenderer::new(MapMode::Continuous, self)
    }

    /// Fallible variant of
    /// [`build_continuous_renderer`](Self::build_continuous_renderer); see
    /// [`try_build_static_renderer`](Self::try_build_static_renderer).
    ///
    /// # Errors
    ///
    /// Returns [`RendererBuildError`] when the native renderer cannot be
    /// constructed.
    pub fn try_build_continuous_renderer(
        self,
    ) -> Result<ImageRenderer<Continuous>, RendererBuildError> {
        ImageRenderer::try_new(MapMode::Continuous, self)
    }
}

/// The native renderer could not be constructed.
///
/// Construction reaches the platform graphics stack (context creation inside
/// the headless frontend), so a broken display environment fails here rather
/// than at render time. The message carries the native exception's text.
#[derive(thiserror::Error, Debug)]
#[error("failed to construct the native map renderer: {0}")]
pub struct RendererBuildError(String);

impl<S> ImageRenderer<S> {
    /// Creates a new renderer instance
    fn new(map_mode: MapMode, opts: ImageRendererBuilder) -> Self {
        Self::try_new(map_mode, opts).expect("native map renderer construction failed")
    }

    /// Creates a new renderer instance, surfacing native construction
    /// exceptions as [`RendererBuildError`] instead of terminating the
    /// process.
    fn try_new(map_mode: MapMode, opts: ImageRendererBuilder) -> Result<Self, RendererBuildError> {
        let resource_options = opts.resource_options.unwrap_or_default();
        let mut map = ffi::MapRenderer_new(
            map_mode,
            opts.width.get(),
            opts.height.get(),
            opts.pixel_ratio,
            resource_options.as_ref(),
        )
        .map_err(|exception| RendererBuildError(exception.what().to_string()))?;

        // Wire up the observer dispatchers once; `map_observer()` afterwards is
        // a pure view that only swaps the stored callbacks.
        let observer_callbacks = Rc::new(MapObserverCallbacks::default());
        observer_callbacks.install(&map.pin_mut().observer());

        Ok(Self {
            instance: map,
            observer_callbacks,
            style_specified: false,
            _marker: PhantomData,
            _not_send: PhantomData,
        })
    }
}
