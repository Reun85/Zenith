// #![allow(dead_code)]
// #[repr(C)]
// pub union WithMinByteSize<T: Copy, const N: usize> {
//     t: T,
//     _marker: [u8; N],
// }
//
// impl<T: Copy, const N: usize> WithMinByteSize<T, N> {
//     #[inline]
//     pub fn new(t: T) -> Self {
//         WithMinByteSize { t }
//     }
// }
//
// impl<T: Copy, const N: usize> std::ops::Deref for WithMinByteSize<T, N> {
//     type Target = T;
//     #[inline]
//     fn deref(&self) -> &Self::Target {
//         unsafe { &self.t }
//     }
// }
//
// impl<T: Copy, const N: usize> std::ops::DerefMut for WithMinByteSize<T, N> {
//     #[inline]
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         unsafe { &mut self.t }
//     }
// }
//
// impl<T: Copy + std::fmt::Debug, const N: usize> std::fmt::Debug for WithMinByteSize<T, N> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "AlignAs {} bytes : {:?} ", N, unsafe { self.t })
//     }
// }
//
// // N must be a multiple of 4
// // Or it will not work as intended.
// pub type PadToAlign<T, const N: usize> =
//     WithMinByteSize<T, { get_align_min_size(std::mem::size_of::<T>(), N) }>;
// pub const fn get_align_min_size(actual: usize, align: usize) -> usize {
//     assert!(align % 4 == 0, "N must be a multiple of 4");
//     (actual / align + (actual % align != 0) as usize) * align
// }
// #[cfg(test)]
// mod test {
//     use super::*;
//     use cgmath::*;
//     #[repr(C)]
//     struct Test {
//         a: PadToAlign<Matrix4<f32>, 16>,
//         b: PadToAlign<Vector3<f32>, 16>,
//         c: PadToAlign<Vector3<f32>, 16>,
//         d: PadToAlign<Matrix4<f32>, 16>,
//     }
//     #[test]
//     fn test1() {
//         assert_eq!(0, std::mem::offset_of!(Test, a),);
//         assert_eq!(64, std::mem::offset_of!(Test, b),);
//         assert_eq!(80, std::mem::offset_of!(Test, c),);
//         assert_eq!(96, std::mem::offset_of!(Test, d),);
//         assert_eq!(160, std::mem::size_of::<Test>());
//
//         // It rounds up to 4s.
//         assert_eq!(68, std::mem::size_of::<WithMinByteSize<Matrix4<f32>, 65>>());
//         assert_eq!(80, std::mem::size_of::<PadToAlign<Matrix4<f32>, 20>>());
//     }
// }
