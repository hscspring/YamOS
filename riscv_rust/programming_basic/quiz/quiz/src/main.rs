// macro_rules! m {
//     ($($s:stmt)*) => {
//         $(
//             { stringify!($s); 2 }
//         )<<*
//     };
// }
// fn main() {
//     print!(
//         "{}{}{}",
//         m! { return || true },
//         m! { (return) || true },
//         m! { {return} || true },
//     );
// }

// macro_rules! m {
//     (==>) => {
//         print!("1");
//     };
//     (= = >) => {
//         print!("2");
//     };
//     (== >) => {
//         print!("3");
//     };
//     (= =>) => {
//         print!("4");
//     };
// }

// fn main() {
//     m!(==>);
//     m!(= = >);
//     m!(== >);
//     m!(= =>);
// }

// fn f<'a>() {}
// fn g<'a: 'a>() {}

// fn main() {
//     let pf = f::<'static> as fn();
//     let pg = g::<'static> as fn();
//     print!("{}", pf == pg);
// }

// fn main() {
//     let input = vec![1, 2, 3];

//     let parity = input.iter().map(|x| {
//         print!("{}", x);
//         x % 2
//     });

//     for p in parity {
//         print!("{}", p);
//     }
// }

// use std::mem;

// fn main() {
//     let a;
//     let a = a = true;
//     print!("{:?}", a);
//     print!("{}", mem::size_of_val(&a));
// }

fn main() {
    let x: u8 = 1;
    const K: u8 = 2;

    macro_rules! m {
        () => {
            print!("{}{}", x, K);
        };
    }

    {
        let x: u8 = 3;
        const K: u8 = 4;

        m!();
    }
}
