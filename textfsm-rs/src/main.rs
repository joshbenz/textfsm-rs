use std::time::Instant;
use textfsm::template::parse_template;

fn main() {
    let s = r"# Chassis value will be null for single chassis routers.
Value Filldown Chassis (.cc.?-re.)
Value Required Slot (\d+)
Value State (\w+)
Value Temp (\d+)
Value CPUTemp (\d+)
Value DRAM (\d+)
Value Model (\S+)

adfasdf
";
    let start = Instant::now();
    let res = parse_template(s);
    let end = start.elapsed();

    println!("LOOPS Took {:?} micros", end.as_micros());
    println!("{:?}", res);
}
