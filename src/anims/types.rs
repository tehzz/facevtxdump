use std::fmt;

/// 1 : Mat4 (f32[4][4])
/// 6 : s16[3]
/// 7 : s16[3]
/// 8 : s16[6]
/// 3 : s16[9] -> gets converted to type 9 by alloc_animdata
/// 11: s16[6]
/// 2 : f32[3][3] (GdTriangleF)
/// 9 : {Mat4, Vec3f{x,y,z}}
/// 4 : f32[3][3] (GdTriangleF)
/// 5 : Stubbed
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnimKind {
    Empty = 0,
    Matrix = 1,
    Triangle_2 = 2,
    Short9 = 3,
    Triangle_4 = 4,
    Stub = 5,
    Short3_6 = 6,
    Short3_7 = 7,
    Short6_8 = 8,
    MatVec = 9,
    Short6_11 = 11,
}
impl AnimKind {
    pub fn from_i32(int: i32) -> Self {
        use self::AnimKind::*;
        match int {
            0  => Empty,
            1  => Matrix,
            2  => Triangle_2,
            3  => Short9,
            4  => Triangle_4,
            5  => Stub,
            6  => Short3_6,
            7  => Short3_7,
            8  => Short6_8,
            9  => MatVec,
            11 => Short6_11,
            _ => panic!("Unknown Animation type {}", int),
        }
    }
}
impl fmt::Display for AnimKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::AnimKind::*;
        let constant = match self {
            Empty => "GD_ANIM_EMPTY",
            Matrix => "GD_ANIM_MATRIX",
            Triangle_2 => "GD_ANIM_TRI_F_2",
            Short9 => "GD_ANIM_9H",
            Triangle_4 => "GD_ANIM_TRI_F_4",
            Stub => "GD_ANIM_STUB",
            Short3_6 => "GD_ANIM_3H_SCALED",
            Short3_7 => "GD_ANIM_3H",
            Short6_8 => "GD_ANIM_6H_SCALED",
            MatVec => "GD_ANIM_MTX_VEC",
            Short6_11 => "GD_ANIM_CAMERA",
        };
        write!(f, "{}", constant)
    }
}

/* 4x4 float matrix */
#[derive(Debug, Clone)]
pub struct Mtx([[f32; 4]; 4]);
impl Mtx { 
    pub const SIZE: usize = 4 * 4; 
}
impl<'a> From<&'a [f32]> for Mtx {
    fn from(buf: &[f32]) -> Self {
        assert!(buf.len() > 16);
        let mut o = [[0.0; 4]; 4];
        for (i, row) in buf.chunks(4).take(4).enumerate() {
            o[i][0] = row[0];
            o[i][1] = row[1];
            o[i][2] = row[2];
            o[i][3] = row[3];
        }
        Mtx(o)
    }
}
impl fmt::Display for Mtx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = self.0;
        let width = f.width().unwrap_or(0);
        write!(f, "{{ \
            {{ {:w$?}, {:w$?}, {:w$?}, {:w$?} }} \
            {{ {:w$?}, {:w$?}, {:w$?}, {:w$?} }} \
            {{ {:w$?}, {:w$?}, {:w$?}, {:w$?} }} \
            {{ {:w$?}, {:w$?}, {:w$?}, {:w$?} }} \
            }}",
            m[0][0], m[0][1], m[0][2], m[0][3],
            m[1][0], m[1][1], m[1][2], m[1][3],
            m[2][0], m[2][1], m[2][2], m[2][3],
            m[3][0], m[3][1], m[3][2], m[3][3],
            w = width
        )
    }
}