#[cfg(feature = "cairo")]
extern crate cairo;
#[cfg(feature = "cairo")]
pub type RenderCallback = fn(i32, &mut crate::Stream, &crate::PositionType, f32, f32, u32, u32);

#[cfg(feature = "opengl")]
pub type RenderCallback = fn(i32, &mut crate::Stream, &crate::PositionType, f32, f32, u32, u32);

#[cfg(feature = "open")]
pub type RenderCallback = fn(i32, &mut crate::Stream, &crate::PositionType, f32, f32, u32, u32);
