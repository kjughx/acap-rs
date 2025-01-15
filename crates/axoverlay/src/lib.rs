#![allow(dead_code)]

mod backend;
mod error;
mod overlay;
mod settings;

pub use error::Error;
pub use overlay::{Overlay, OverlayBuilder};
pub use settings::Settings;

use std::ffi::c_void;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum StreamType {
    Jpeg,
    H264,
    H265,
    Ycbcr,
    Vout,
    Other,
    Rgb,
    Av1,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum AnchorPoint {
    TopLeft,
    Center,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum PositionType {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    CustomNormalized,
    CustomSource,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Colorspace {
    Arg32,
    Palette4bit,
    Palette1bit,
    Undefined,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Zpriority {
    Low = axoverlay_sys::AXOVERLAY_Z_PRIO_LOW,
    Medium = axoverlay_sys::AXOVERLAY_Z_PRIO_MEDIUM,
    High = axoverlay_sys::AXOVERLAY_Z_PRIO_HIGH,
    VeryHigh = axoverlay_sys::AXOVERLAY_Z_PRIO_VERY_HIGH,
}

pub type SelectCallback = fn(i32, i32, i32, i32, i32, axoverlay_sys::axoverlay_stream_type) -> i32;
pub type AdjustmentCallback =
    fn(i32, &mut Stream, &mut PositionType, &mut f32, &mut f32, &mut u32, &mut u32);
pub type SyncedRenderCallback =
    fn(i32, &mut Stream, &PositionType, f32, f32, u32, u32, &axoverlay_sys::timeval);

pub type RenderCallback = backend::RenderCallback;

pub enum Callback {
    Adjustment(AdjustmentCallback),
    Render(RenderCallback),
    RenderSynced(SyncedRenderCallback),
    Select(SelectCallback),
}

pub struct Stream(axoverlay_sys::axoverlay_stream_data);
impl Stream {
    pub fn from_raw<'a>(stream: *mut axoverlay_sys::axoverlay_stream_data) -> &'a mut Self {
        assert!(!stream.is_null());
        unsafe { &mut (*(stream as *mut Self)) }
    }
}

pub struct Palette {
    inner: *mut axoverlay_sys::axoverlay_palette_color,
}

impl Palette {
    fn from_raw(palette: *mut axoverlay_sys::axoverlay_palette_color) -> Self {
        assert!(!palette.is_null());
        return Self { inner: palette };
    }
}

pub struct Axoverlay(Settings);
impl Axoverlay {
    pub fn new(settings: Settings) -> Result<Self, Error> {
        let this = Self(settings);
        match this.init() {
            Ok(_) => Ok(this),
            Err(e) => Err(e),
        }
    }

    pub fn enable_cpu_mem_sync(sync: bool) {
        unsafe { axoverlay_sys::axoverlay_enable_cpu_mem_sync(sync as i32) };
    }

    pub fn settings(&mut self) -> &mut Settings {
        &mut self.0
    }

    fn init(&self) -> Result<(), Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        unsafe { axoverlay_sys::axoverlay_init(self.0.into_raw(), &mut error as *mut _) }
        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(())
    }

    pub fn destroy(self) {
        unsafe { axoverlay_sys::axoverlay_cleanup() }
    }

    pub fn reload_streams(&self) -> Result<(), Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        unsafe { axoverlay_sys::axoverlay_reload_streams(&mut error as *mut _) };
        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(())
    }

    pub fn redraw(&self) -> Result<(), Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        unsafe { axoverlay_sys::axoverlay_redraw(&mut error as *mut _) };
        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(())
    }

    pub fn create_overlay(&self, builder: Option<OverlayBuilder>) -> Result<Overlay, Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        let mut overlaydata = if let Some(ref builder) = builder {
            builder.into_inner()
        } else {
            OverlayBuilder::default().into_inner()
        };

        let id = unsafe {
            axoverlay_sys::axoverlay_create_overlay(
                &mut overlaydata as *mut _,
                &self.0 as *const _ as *mut c_void, // TODO: Add user_data
                &mut error as *mut _,
            )
        };
        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(Overlay {
            id: Some(id),
            inner: Box::into_raw(Box::new(overlaydata)),
            builder,
        })
    }

    pub fn destroy_overlay(&self, id: i32) -> Result<(), Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        unsafe { axoverlay_sys::axoverlay_destroy_overlay(id, &mut error as *mut _) };
        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(())
    }

    pub fn set_overlay_position(
        id: i32,
        postype: PositionType,
        x: f32,
        y: f32,
    ) -> Result<(), Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        unsafe {
            axoverlay_sys::axoverlay_set_overlay_position(
                id,
                postype as u32,
                x,
                y,
                &mut error as *mut _,
            )
        };

        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(())
    }

    pub fn set_overlay_size(id: i32, width: i32, height: i32) -> Result<(), Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        unsafe {
            axoverlay_sys::axoverlay_set_overlay_size(id, width, height, &mut error as *mut _)
        };

        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(())
    }

    pub fn get_max_resolution(&self, camera: i32) -> Result<(i32, i32), Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        let width = unsafe {
            axoverlay_sys::axoverlay_get_max_resolution_width(camera, &mut error as *mut _)
        };
        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        let height = unsafe {
            axoverlay_sys::axoverlay_get_max_resolution_height(camera, &mut error as *mut _)
        };
        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok((width, height))
    }

    pub fn get_number_of_palette_colors(&self) -> Result<i32, Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        let n =
            unsafe { axoverlay_sys::axoverlay_get_number_of_palette_colors(&mut error as *mut _) };

        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(n)
    }

    pub fn get_palette_color(idx: i32) -> Result<Palette, Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        let palette: *mut axoverlay_sys::axoverlay_palette_color = std::ptr::null_mut();
        unsafe { axoverlay_sys::axoverlay_get_palette_color(idx, palette, &mut error as *mut _) };

        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(Palette::from_raw(palette))
    }

    pub fn set_palette_color(idx: i32, palette: &Palette) -> Result<(), Error> {
        let mut error: *mut glib_sys::GError = std::ptr::null_mut();
        unsafe {
            axoverlay_sys::axoverlay_set_palette_color(idx, palette.inner, &mut error as *mut _)
        };

        if !error.is_null() {
            return Err(Error::from_raw(error));
        }

        Ok(())
    }
}
