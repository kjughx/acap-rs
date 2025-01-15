use crate::*;

#[derive(Default)]
pub struct OverlayBuilder {
    postype: Option<PositionType>,
    anchor_point: Option<AnchorPoint>,
    x: Option<f32>,
    y: Option<f32>,
    width: Option<i32>,
    height: Option<i32>,
    colorspace: Option<Colorspace>,
    z_priority: Option<Zpriority>,
    scale_to_stream: Option<bool>,
}

impl OverlayBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn postype(mut self, p: PositionType) -> Self {
        self.postype = Some(p);
        self
    }
    pub fn anchor_point(mut self, a: AnchorPoint) -> Self {
        self.anchor_point = Some(a);
        self
    }
    pub fn x(mut self, x: f32) -> Self {
        self.x = Some(x);
        self
    }
    pub fn y(mut self, y: f32) -> Self {
        self.y = Some(y);
        self
    }
    pub fn width(mut self, w: i32) -> Self {
        self.width = Some(w);
        self
    }
    pub fn height(mut self, h: i32) -> Self {
        self.height = Some(h);
        self
    }
    pub fn colorspace(mut self, c: Colorspace) -> Self {
        self.colorspace = Some(c);
        self
    }
    pub fn z_priority(mut self, zp: Zpriority) -> Self {
        self.z_priority = Some(zp);
        self
    }
    pub fn scale_to_stream(mut self, b: bool) -> Self {
        self.scale_to_stream = Some(b);
        self
    }

    pub(crate) fn into_inner(&self) -> axoverlay_sys::axoverlay_overlay_data {
        let mut raw: std::mem::MaybeUninit<axoverlay_sys::axoverlay_overlay_data> =
            std::mem::MaybeUninit::uninit();
        let mut this = unsafe {
            axoverlay_sys::axoverlay_init_overlay_data(raw.as_mut_ptr());
            raw.assume_init()
        };

        if let Some(p) = self.postype {
            this.postype = p as u32;
        }
        if let Some(a) = self.anchor_point {
            this.anchor_point = a as u32;
        }
        if let Some(x) = self.x {
            this.x = x;
        }
        if let Some(y) = self.y {
            this.y = y;
        }
        if let Some(w) = self.width {
            this.width = w;
        }
        if let Some(h) = self.height {
            this.height = h;
        }
        if let Some(c) = self.colorspace {
            this.colorspace = c as u32;
        }
        if let Some(zp) = self.z_priority {
            this.z_priority = zp as i32;
        }
        if let Some(b) = self.scale_to_stream {
            this.scale_to_stream = b as i32;
        }

        this
    }
}

pub struct Overlay {
    pub(crate) id: Option<i32>,
    pub(crate) inner: *mut axoverlay_sys::axoverlay_overlay_data,
    pub(crate) builder: Option<OverlayBuilder>,
}

impl Overlay {
    pub fn from_raw(overlay: *mut axoverlay_sys::axoverlay_overlay_data) -> Self {
        assert!(!overlay.is_null());
        Self {
            id: None,
            inner: overlay,
            builder: None,
        }
    }

    fn as_mut(&mut self) -> &mut axoverlay_sys::axoverlay_overlay_data {
        unsafe { self.inner.as_mut().unwrap() }
    }
}

impl Default for Overlay {
    fn default() -> Self {
        let inner = std::ptr::null_mut();
        unsafe { axoverlay_sys::axoverlay_init_overlay_data(inner) };

        Self::from_raw(inner)
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        let Some(id) = self.id else {
            return;
        };

        unsafe { axoverlay_sys::axoverlay_destroy_overlay(id, std::ptr::null_mut()) }
    }
}
