#![allow(dead_code)]
extern crate glsl_to_spirv_macro;

glsl_to_spirv_macro::shader! {
    shaders: [
        {
            name: vert,
            ty: "frag",
            path: "tests/shaders/vert.vert"
        },
    ],
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn compiled2() {}
    #[test]
    fn compiled() {
        let _x = vert::load_words();
        let _y = vert::Input {
            normal: [1.0, 2.0, 3.0].into(),
            position: [1.0, 2.0, 3.0].into(),
            texcoord: [1.0, 2.0].into(),
        };
    }
}
