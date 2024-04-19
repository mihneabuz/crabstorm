mod node;
mod op;

use crabstorm::*;

fn main() {
    Runtime::new().run(node::KvNode::new()).unwrap()
}
