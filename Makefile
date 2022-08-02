# default features are play and solve
FEATURES := "play,solve"

# force rebuild, because cargo is smart and FEATURES might be different
.PHONY: target/release/hustle
target/release/hustle: $(shell find src)
	cargo build --release --features $(FEATURES)

.PHONY: test
test:
	cargo test --release --features "play,solve,gen"

.PHONY: install
install: target/release/hustle
	# binary
	sudo install -Dm0755 -t "/usr/bin" "target/release/hustle"
	# data
	sudo install -Dm0644 -t "/usr/share/hustle" "data/bank1.csv"
	sudo install -Dm0644 -t "/usr/share/hustle" "data/bank2.csv"
	sudo install -Dm0644 -t "/usr/share/hustle" "data/happrox.csv"
	sudo install -Dm0644 -t "/usr/share/hustle" "data/lbounds.csv"
	sudo install -Dm0644 -t "/usr/share/hustle" "data/config.toml"
	# manpages
	sudo install -Dm0644 -t "/usr/share/man/man1" "extra/manpages/hustle.1"
	sudo install -Dm0644 -t "/usr/share/man/man1" "extra/manpages/hustle-solve.1"
	sudo install -Dm0644 -t "/usr/share/man/man1" "extra/manpages/hustle-play.1"
	sudo install -Dm0644 -t "/usr/share/man/man1" "extra/manpages/hustle-hgen.1"
	sudo install -Dm0644 -t "/usr/share/man/man1" "extra/manpages/hustle-ggen.1"
	sudo install -Dm0644 -t "/usr/share/man/man1" "extra/manpages/hustle-lgen.1"
	# misc
	sudo install -Dm0644 -t "/usr/share/licenses/hustle" "LICENSE"
	sudo install -Dm0644 -t "/usr/share/doc/hustle" "README.md"

.PHONY: uninstall
uninstall:
	# binary
	sudo rm -rf "/usr/bin/hustle"
	# data
	sudo rm -rf "/usr/share/hustle"
	# manpages
	sudo rm -rf "/usr/share/man/man1/hustle.1"
	sudo rm -rf "/usr/share/man/man1/hustle-solve.1"
	sudo rm -rf "/usr/share/man/man1/hustle-play.1"
	sudo rm -rf "/usr/share/man/man1/hustle-hgen.1"
	sudo rm -rf "/usr/share/man/man1/hustle-ggen.1"
	sudo rm -rf "/usr/share/man/man1/hustle-lgen.1"
	# misc
	sudo rm -rf "/usr/share/licenses/hustle"
	sudo rm -rf "/usr/share/doc/hustle"
