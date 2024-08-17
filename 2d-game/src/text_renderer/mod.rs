extern crate nalgebra_glm as glm;

pub mod ascii;
pub mod utf8;

pub trait ITextRenderer {
    fn render_text(
        &self,
        text: String,
        x: f32,
        y: f32,
        scale: f32
    );

    fn render_text_ex(
        &self,
        text: String,
        x: f32,
        y: f32,
        scale: f32,
        color: glm::TVec3<f32>
    );
}