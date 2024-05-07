echo-bin:
	cargo build --bin echo

echo: echo-bin
	./maelstrom/maelstrom test -w echo --bin ./target/debug/echo --node-count 10 --time-limit 10


unique-bin:
	cargo build --bin unique

unique: unique-bin
	./maelstrom/maelstrom test -w unique-ids --bin ./target/debug/unique --time-limit 30 --rate 1000 --node-count 5 --availability total --nemesis partition


linkv-bin:
	cargo build --bin unique

linkv: linkv-bin
	./maelstrom/maelstrom test -w lin-kv --bin ./target/debug/linkv --time-limit 10 --node-count 1 --concurrency 2n


broadcast-bin:
	cargo build --bin broadcast

broadcast: broadcast-bin
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 5 --time-limit 20 --rate 10


counter-bin:
	cargo build --bin counter

counter: counter-bin
	./maelstrom/maelstrom test -w g-counter --bin ./target/debug/counter --node-count 5 --rate 100 --time-limit 20 --nemesis partition


set-bin:
	cargo build --bin set

set: set-bin
	./maelstrom/maelstrom test -w g-set --bin ./target/debug/set --time-limit 10 --rate 100 --nemesis partition


transactkv-bin:
	cargo build --bin transactkv

transactkv: transactkv-bin
	./maelstrom/maelstrom test -w txn-list-append --bin ./target/debug/transactkv --time-limit 10 --node-count 1 --concurrency 10n --rate 100


logs-bin:
	cargo build --bin logs

logs: logs-bin
	./maelstrom/maelstrom test -w kafka --bin ./target/debug/logs --node-count 1 --concurrency 2n --time-limit 20 --rate 1000


all: echo unique broadcast counter set logs


serve:
	./maelstrom/maelstrom serve

download:
	wget https://github.com/jepsen-io/maelstrom/releases/download/v0.2.3/maelstrom.tar.bz2
	tar -xvf maelstrom.tar.bz2
	rm maelstrom.tar.bz2
