// macro_rules! splat {
//     attr() ($visibility:vis fn $identifier:ident(
//         $(
//             $parameter:tt
//         )*
//     ) $($other:tt)*) => {
//         #[cfg(not(rust_analyzer))]
//         $visibility fn $identifier($($parameter)*) $($other)*

//         fn sad(self) {}
//     };
// }

// trait Either {}
// impl Either for (i32, i32) {}
// impl Either for (i32,) {}

// struct Tester;

// impl Tester {
//     #[splat]
//     fn bad(self, #[rustc_splat] _: impl Either) {}
// }

// fn tester() {
//     Tester.bad(0);
//     //Tester.bad(0, 0);
//     //Tester.bad(0, 0);
//     Tester.sad();
// }

// impl Tester {
//     #[expect(unexpected_cfgs)]
//     {
//         #[cfg(rust_analyzer)]
//     fn blah() {}
//     }
// }
