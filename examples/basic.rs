use varies::varies;

#[varies]
fn test_functions() -> i32 {
    let mut a = 1;
    #[variant(extra)]
    a += 1;

    #[variant(hi)]
    {
        println!(":)")
    };

    #[variant(timing)]
    {
        let t = 1;
    };

    let b = 1;
    a
}

fn main() {
    test_functions::hi();
    test_functions::hi();
    test_functions::timing();

    println!("{}", test_functions::default());
    println!("{}", test_functions::extra());
}
