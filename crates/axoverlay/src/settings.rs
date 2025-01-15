use crate::{
    AdjustmentCallback, Callback, PositionType, RenderCallback, SelectCallback, Stream,
    SyncedRenderCallback,
};
use log::error;

macro_rules! suppress_unwind {
    ($f:expr) => {
        match std::panic::catch_unwind($f) {
            Ok(r) => Ok(r),
            Err(e) => match e.downcast::<String>() {
                Ok(e) => {
                    error!("Caught panic in callback (string) {e}");
                    Err(())
                }
                Err(e) => {
                    error!("Caught panic in callback (other) {e:?}");
                    Err(())
                }
            },
        }
    };
}

pub struct Settings {
    inner: *mut axoverlay_sys::axoverlay_settings,
    adjustment_cb: Option<AdjustmentCallback>,
    render_cb: Option<RenderCallback>,
    synced_render_cb: Option<SyncedRenderCallback>,
}

// Ideally this would be part of Settings, but since `axoverlay_set_stream_select_callback` doesn't
// provide a `user_data` it would not be possible to locate the provided closure. Instead we store
// it here as a thread_local so that it can be accessed from Settings::select_cb
thread_local! {
    static SELECT_CB: std::cell::Cell<Option<SelectCallback>> = std::cell::Cell::new(None);
}

impl Default for Settings {
    fn default() -> Self {
        let settings = std::ptr::null_mut();
        unsafe { axoverlay_sys::axoverlay_init_axoverlay_settings(settings) };
        Self {
            inner: settings,
            adjustment_cb: None,
            render_cb: None,
            synced_render_cb: None,
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_callback(self, callback: Callback) -> Self {
        let mut this = self;
        match callback {
            Callback::Adjustment(f) => unsafe {
                this.adjustment_cb = Some(f);
                axoverlay_sys::axoverlay_set_adjustment_callback(Some(Self::adjustment_cb))
            },
            Callback::Render(f) => unsafe {
                this.render_cb = Some(f);
                axoverlay_sys::axoverlay_set_render_callback(Some(Self::render_cb))
            },
            Callback::RenderSynced(f) => unsafe {
                this.synced_render_cb = Some(f);
                axoverlay_sys::axoverlay_set_synced_render_callback(Some(Self::render_synced_cb))
            },
            Callback::Select(f) => unsafe {
                SELECT_CB.with(|cb| cb.replace(Some(f)));
                axoverlay_sys::axoverlay_set_stream_select_callback(Some(Self::select_cb))
            },
        }

        this
    }

    pub(crate) fn into_raw(&self) -> *mut axoverlay_sys::axoverlay_settings {
        self.inner
    }

    #[allow(unused_must_use)]
    unsafe extern "C" fn adjustment_cb(
        id: axoverlay_sys::gint,
        stream: *mut axoverlay_sys::axoverlay_stream_data,
        postype: *mut axoverlay_sys::axoverlay_position_type,
        overlay_x: *mut axoverlay_sys::gfloat,
        overlay_y: *mut axoverlay_sys::gfloat,
        overlay_width: *mut axoverlay_sys::gint,
        overlay_height: *mut axoverlay_sys::gint,
        user_data: glib_sys::gpointer,
    ) {
        suppress_unwind!(|| {
            let this = &*(user_data as *const Settings);
            let Some(callback) = this.adjustment_cb else {
                return;
            };

            callback(
                id,
                Stream::from_raw(stream),
                (postype as *mut PositionType).as_mut().unwrap(),
                overlay_x.as_mut().unwrap(),
                overlay_y.as_mut().unwrap(),
                (overlay_width as *mut u32).as_mut().unwrap(),
                (overlay_height as *mut u32).as_mut().unwrap(),
            );
        });
    }

    #[allow(unused_must_use)]
    unsafe extern "C" fn render_cb(
        _rendering_context: glib_sys::gpointer,
        id: axoverlay_sys::gint,
        stream: *mut axoverlay_sys::axoverlay_stream_data,
        postype: axoverlay_sys::axoverlay_position_type,
        overlay_x: axoverlay_sys::gfloat,
        overlay_y: axoverlay_sys::gfloat,
        overlay_width: axoverlay_sys::gint,
        overlay_height: axoverlay_sys::gint,
        user_data: glib_sys::gpointer,
    ) {
        suppress_unwind!(|| {
            let this = &*(user_data as *const Settings);
            let Some(callback) = this.render_cb else {
                return;
            };

            callback(
                id,
                Stream::from_raw(stream),
                (postype as *mut PositionType).as_mut().unwrap(),
                overlay_x,
                overlay_y,
                overlay_width as u32,
                overlay_height as u32,
            )
        });
    }

    #[allow(unused_must_use)]
    unsafe extern "C" fn render_synced_cb(
        _rendering_context: glib_sys::gpointer,
        id: axoverlay_sys::gint,
        stream: *mut axoverlay_sys::axoverlay_stream_data,
        postype: axoverlay_sys::axoverlay_position_type,
        overlay_x: axoverlay_sys::gfloat,
        overlay_y: axoverlay_sys::gfloat,
        overlay_width: axoverlay_sys::gint,
        overlay_height: axoverlay_sys::gint,
        timestamp: *mut axoverlay_sys::timeval,
        user_data: glib_sys::gpointer,
    ) {
        suppress_unwind!(|| {
            let this = &*(user_data as *const Settings);
            let Some(callback) = this.synced_render_cb else {
                return;
            };

            callback(
                id,
                Stream::from_raw(stream),
                (postype as *mut PositionType).as_mut().unwrap(),
                overlay_x,
                overlay_y,
                overlay_width as u32,
                overlay_height as u32,
                timestamp.as_ref().unwrap(),
            )
        });
    }

    unsafe extern "C" fn select_cb(
        camera: axoverlay_sys::gint,
        width: axoverlay_sys::gint,
        height: axoverlay_sys::gint,
        rotation: axoverlay_sys::gint,
        is_mirrored: glib_sys::gboolean,
        type_: axoverlay_sys::axoverlay_stream_type,
    ) -> glib_sys::gboolean {
        suppress_unwind!(move || {
            return SELECT_CB.with(|cb| {
                let callback = cb.get().unwrap();
                callback(camera, width, height, rotation, is_mirrored, type_)
            });
        })
        .unwrap_or(0)
    }
}
