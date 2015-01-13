#![allow(unstable)]

fn fibonacci(n: i32) -> i32 {
    match n {
        0 => 0,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

mod test {
    extern crate test;

    use self::test::Bencher;

    #[test]
    fn fib() {
        assert_eq!(super::fibonacci(7), 13);
    }

    #[bench]
    fn fib_10(b: &mut Bencher) {
        let mut x = 10;
        test::black_box(&mut x);
        b.iter(|| super::fibonacci(x))
    }
}
