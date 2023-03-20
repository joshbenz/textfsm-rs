use textfsm::template::parser::parse_template;
fn main() {
    let s = r"# Chassis value will be null for single chassis routers.
Value Filldown Chassis (.cc.?-re.)
Value Required Slot (\d+)
Value State (\w+)
Value Temp (\d+)
Value CPUTemp (\d+)
Value DRAM (\d+)
Value Model (\S+)

";
    println!("{:?}", parse_template(s))
}
