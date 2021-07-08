use ipnetwork::IpNetwork;

fn main() {
    println!("Hello, world!");
    let mut vec: Vec<IpNetwork> = Vec::new();
    for i in 1..250 {
        let mystr = format!("10.12.0.{}", i);
        let ip_host = mystr.parse().unwrap_or_else(|_| {
            eprintln!("invalid address");
            std::process::exit(1);
        });
        vec.push(ip_host);
    }
    for i in vec.iter() {
        print!("{} ", i);
    }
}
