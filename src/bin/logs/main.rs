mod log;
mod node;

use crabstorm::Runtime;
use node::LogNode;

fn main() {
    Runtime::new().run(LogNode::new()).unwrap()
}
