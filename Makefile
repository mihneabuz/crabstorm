echo:
	./maelstrom/maelstrom test -w echo --bin ./target/debug/echo --node-count 1 --time-limit 10

unique:
	./maelstrom/maelstrom test -w unique-ids --bin ./target/debug/unique --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

download:
	wget https://github.com/jepsen-io/maelstrom/releases/download/v0.2.3/maelstrom.tar.bz2
	tar -xvf maelstrom.tar.bz2
	rm maelstrom.tar.bz2
