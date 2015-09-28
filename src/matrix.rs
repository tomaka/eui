use std::ops;

/// A 3x3 matrix. The data is stored in column-major.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Matrix(pub [[f32; 3]; 3]);

impl Matrix {
    /// Builds an identity matrix, in other words a matrix that has no effect.
    #[inline]
    pub fn identity() -> Matrix {
        Matrix([
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    /// Builds a matrix that will rescale both width and height of a given factor.
    #[inline]
    pub fn scale(factor: f32) -> Matrix {
        Matrix([
            [factor,   0.0 , 0.0],
            [  0.0 , factor, 0.0],
            [  0.0 ,   0.0 , 1.0],
        ])
    }

    /// Builds a matrix that will multiply the width and height by a certain factor.
    #[inline]
    pub fn scale_wh(w: f32, h: f32) -> Matrix {
        Matrix([
            [ w,  0.0, 0.0],
            [0.0,  h,  0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    /// Builds a matrix that will translate the object.
    #[inline]
    pub fn translate(x: f32, y: f32) -> Matrix {
        Matrix([
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [ x,   y,  1.0],
        ])
    }
}

impl ops::Mul for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        let me = self.0;
        let other = other.0;

        let a = me[0][0] * other[0][0] + me[1][0] * other[0][1] + me[2][0] * other[0][2];
        let b = me[0][0] * other[1][0] + me[1][0] * other[1][1] + me[2][0] * other[1][2];
        let c = me[0][0] * other[2][0] + me[1][0] * other[2][1] + me[2][0] * other[2][2];
        let d = me[0][1] * other[0][0] + me[1][1] * other[0][1] + me[2][1] * other[0][2];
        let e = me[0][1] * other[1][0] + me[1][1] * other[1][1] + me[2][1] * other[1][2];
        let f = me[0][1] * other[2][0] + me[1][1] * other[2][1] + me[2][1] * other[2][2];
        let g = me[0][2] * other[0][0] + me[1][2] * other[0][1] + me[2][2] * other[0][2];
        let h = me[0][2] * other[1][0] + me[1][2] * other[1][1] + me[2][2] * other[1][2];
        let i = me[0][2] * other[2][0] + me[1][2] * other[2][1] + me[2][2] * other[2][2];

        Matrix([
            [a, d, g],
            [b, e, h],
            [c, f, i],
        ])
    }
}

impl ops::Mul<[f32; 3]> for Matrix {
    type Output = [f32; 3];

    fn mul(self, other: [f32; 3]) -> [f32; 3] {
        let me = self.0;

        let x = me[0][0] * other[0] + me[1][0] * other[1] + me[2][0] * other[2];
        let y = me[0][1] * other[0] + me[1][1] * other[1] + me[2][1] * other[2];
        let z = me[0][2] * other[0] + me[1][2] * other[1] + me[2][2] * other[2];

        [x, y, z]
    }
}

impl From<[[f32; 3]; 3]> for Matrix {
    fn from(val: [[f32; 3]; 3]) -> Matrix {
        Matrix(val)
    }
}

impl Into<[[f32; 3]; 3]> for Matrix {
    fn into(self) -> [[f32; 3]; 3] {
        self.0
    }
}

impl Into<[[f32; 4]; 4]> for Matrix {
    fn into(self) -> [[f32; 4]; 4] {
        let m = self.0;

        [
            [m[0][0], m[0][1], 0.0, m[0][2]],
            [m[1][0], m[1][1], 0.0, m[1][2]],
            [  0.0,     0.0,   0.0,   0.0  ],
            [m[2][0], m[2][1], 0.0, m[2][2]]
        ]
    }
}
