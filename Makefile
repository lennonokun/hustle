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
	sudo install -Dm0644 -t "/usr/share/hustle" "data/config.toml"
	# manpages
	gzip -c "extra/manpages/hustle.man" \
		| sudo dd status=none of="/usr/share/man/man1/hustle.1.gz"
	gzip -c "extra/manpages/hustle-solve.man" \
		| sudo dd status=none of="/usr/share/man/man1/hustle-solve.1.gz"
	gzip -c "extra/manpages/hustle-play.man" \
		| sudo dd status=none of="/usr/share/man/man1/hustle-play.1.gz"
	# misc
	sudo install -Dm0644 -t "/usr/share/licenses/hustle" "LICENSE"
	sudo install -Dm0644 -t "/usr/share/doc/hustle" "README.md"

uninstall:
	# binary
	sudo rm "/usr/bin/hustle"
	# data
	sudo rm -rf "/usr/share/hustle"
	# manpages
	sudo rm -rf "/usr/share/man/man1/hustle.1.gz"
	sudo rm -rf "/usr/share/man/man1/hustle-solve.1.gz"
	sudo rm -rf "/usr/share/man/man1/hustle-play.1.gz"
	# misc
	sudo rm -rf "/usr/share/licenses/hustle"
	sudo rm -rf "/usr/share/doc/hustle"
