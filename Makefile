echo:
	./maelstrom/maelstrom test -w echo --bin ./target/debug/echo --node-count 1 --time-limit 10

unique:
	./maelstrom/maelstrom test -w unique-ids --bin ./target/debug/unique --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast-single:
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 1 --time-limit 20 --rate 10

broadcast-multi:
	./maelstrom/maelstrom test -w broadcast --bin ./target/debug/broadcast --node-count 5 --time-limit 20 --rate 10

all: echo unique broadcast-single broadcast-multi

serve:
	./maelstrom/maelstrom serve

download:
	wget https://github.com/jepsen-io/maelstrom/releases/download/v0.2.3/maelstrom.tar.bz2
	tar -xvf maelstrom.tar.bz2
	rm maelstrom.tar.bz2
