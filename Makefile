target = debug

test_flags = --time-limit 10

ifneq ($(target), debug)
	build_flags = --$(target)
	test_flags = --time-limit 30
endif

ifeq ($(log), true)
	test_flags += --log-stderr
endif

echo-bin:
	cargo build $(build_flags) --bin echo

echo: echo-bin
	./maelstrom/maelstrom test -w echo --bin ./target/$(target)/echo $(test_flags) --node-count 10 --rate 1000 --availability total --nemesis partition


unique-bin:
	cargo build $(build_flags) --bin unique

unique: unique-bin
	./maelstrom/maelstrom test -w unique-ids --bin ./target/$(target)/unique $(test_flags) --rate 1000 --node-count 5 --availability total --nemesis partition


broadcast-bin:
	cargo build $(build_flags) --bin broadcast

broadcast: broadcast-bin
	./maelstrom/maelstrom test -w broadcast --bin ./target/$(target)/broadcast $(test_flags) --node-count 5 --rate 50 --nemesis partition


counter-bin:
	cargo build $(build_flags) --bin counter

counter: counter-bin
	./maelstrom/maelstrom test -w g-counter --bin ./target/$(target)/counter $(test_flags) --node-count 5 --rate 100 --nemesis partition


set-bin:
	cargo build $(build_flags) --bin set

set: set-bin
	./maelstrom/maelstrom test -w g-set --bin ./target/$(target)/set $(test_flags) --node-count 5 --rate 200 --nemesis partition


linkv-bin:
	cargo build $(build_flags) --bin linkv

linkv: linkv-bin
	./maelstrom/maelstrom test -w lin-kv --bin ./target/$(target)/linkv $(test_flags) --node-count 5 --rate 1 --concurrency 2n


transactkv-bin:
	cargo build $(build_flags) --bin transactkv

transactkv: transactkv-bin
	./maelstrom/maelstrom test -w txn-list-append --bin ./target/$(target)/transactkv $(test_flags) --node-count 1 --concurrency 10n --rate 100


logs-bin:
	cargo build $(build_flags) --bin logs

logs: logs-bin
	./maelstrom/maelstrom test -w kafka --bin ./target/$(target)/logs $(test_flags) --node-count 1 --concurrency 2n --rate 1000


all: echo unique broadcast counter set


serve:
	./maelstrom/maelstrom serve

download:
	wget https://github.com/jepsen-io/maelstrom/releases/download/v0.2.3/maelstrom.tar.bz2
	tar -xvf maelstrom.tar.bz2
	rm maelstrom.tar.bz2
