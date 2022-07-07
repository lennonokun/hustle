target/release/hustle: $(shell find src)
	cargo build -r

# is there a better way than repeating sudo
install: target/release/hustle
	# binary
	sudo install -Dm0755 -t "/usr/bin" "target/release/hustle"
	# data
	sudo install -Dm0644 -t "/usr/share/hustle" "data/bank1.csv"
	sudo install -Dm0644 -t "/usr/share/hustle" "data/bank2.csv"
	sudo install -Dm0644 -t "/usr/share/hustle" "data/happrox.csv"
	# misc
	sudo install -Dm0644 -t "/usr/share/licenses/hustle" "LICENSE"
	sudo install -Dm0644 -t "/usr/share/doc/hustle" "README.md"

uninstall:
	# binary
	sudo rm "/usr/bin/hustle"
	# data
	sudo rm -rf "/usr/share/hustle"
	# misc
	sudo rm -rf "/usr/share/licenses/hustle"
	sudo rm -rf "/usr/share/doc/hustle"
